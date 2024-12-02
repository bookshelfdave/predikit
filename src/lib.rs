// Copyright (c) 2025 Dave Parfitt

use self::predikit::data::events::ChkLifecycleEvent;
use clap::Parser;
use codespan_reporting::files::SimpleFiles;
use lalrpop_util::lexer::Token;
use lalrpop_util::ParseError;
use log::debug;
use predikit::comp::ast::{AstFile, AstFileChecks, AstFileTools};
use predikit::comp::errors::{show_fancy_compile_errors, show_fancy_error};
use predikit::comp::tokens::LexicalError;
use predikit::comp::{pkparser, CompiledCheckFileOut};
use predikit::data::events::{desc_from_instances, ChkDescMap};
use predikit::data::instance::{ChkInstance, RunEnv};
use predikit::data::ChkDefRegistry;
use predikit::formatters::default::DefaultOutputFormatter;
use predikit::formatters::FormatterConfig;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

pub mod predikit;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// list of input files
    #[arg(value_name = "FILES")]
    infiles: Vec<PathBuf>,

    /// Show color output
    #[arg(long, short, action)]
    no_color: bool,

    /// Parse input files without running checks
    #[arg(long, short, action)]
    parse_only: bool,

    /// Enable debug logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

fn spawn_listener(
    listener_config: FormatterConfig,
) -> (
    std::sync::mpsc::Sender<ChkLifecycleEvent>,
    thread::JoinHandle<()>,
) {
    let (tx, rx) = channel();
    let iot = thread::spawn(move || {
        let mut listener = DefaultOutputFormatter::new(listener_config);
        listener.process_events(rx);
    });
    (tx, iot)
}

fn load_source_from_file(filename: &PathBuf) -> Result<String, String> {
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            return Err(format!(
                "Can't open {}, skipping: {}",
                filename.to_string_lossy(),
                e
            ))
        }
    };

    let mut source = String::new();

    if let Err(e) = file.read_to_string(&mut source) {
        return Err(format!(
            "Can't read {}, skipping: {}",
            filename.to_string_lossy(),
            e
        ));
    }
    Ok(source)
}

// fn write_parse_tree(ast: &AstFile) -> std::io::Result<()> {
//     use std::io::Write;
//     let mut file = File::create("./parse.out")?;
//     write!(file, "{:#?}", ast)?;
//     Ok(())
// }

fn write_compiled_ast(ast: &Vec<CompiledCheckFileOut<'_>>) -> std::io::Result<()> {
    use std::io::Write;
    let mut file = File::create("./ast.out")?;
    write!(file, "{:#?}", ast)?;
    Ok(())
}

pub fn process_errors(cfas: &Vec<CompiledCheckFileOut>) -> bool {
    let mut errors_found = false;
    for cfa in cfas {
        // typechecking errors etc
        if !cfa.errors.is_empty() {
            errors_found = true;
            let source = match load_source_from_file(&PathBuf::from(
                cfa.filename.clone().unwrap_or_default(),
            )) {
                Ok(source) => source,
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            };
            for err in &cfa.errors {
                show_fancy_error(
                    &err.content_address,
                    &err.message,
                    "",
                    &source,
                    &cfa.filename,
                );
            }
        }
    }
    errors_found
}

// return bool is the final result of running all checks
pub fn process_cli() -> bool {
    let cli = Cli::parse();

    let log_level = match cli.debug {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Info,
        _ => log::LevelFilter::Debug,
    };
    env_logger::builder().filter_level(log_level).init();
    debug!("Log level: {}", log_level);

    if cli.infiles.is_empty() {
        println!("No input files specified");
        return false;
    }

    let listener_config = FormatterConfig {
        color: !cli.no_color,
    };

    // compilation starts here
    let ast_files = parse_files(&cli);
    if let None = ast_files {
        //println!("Fatal compilation error");
        return false;
    }

    let (ast_file_checks, ast_file_tools) = ast_files.unwrap();

    let mut fns = ChkDefRegistry::new_with_builtins();
    let tool_errors = crate::predikit::comp::compiler::make_tools(&mut fns, ast_file_tools);
    if !tool_errors.is_empty() {
        show_fancy_compile_errors(tool_errors);
        return false;
    }

    let cfas = compile_checks(ast_file_checks, &fns);
    if process_errors(&cfas) {
        return false;
    }

    if log_level == log::LevelFilter::Debug {
        let _ = write_compiled_ast(&cfas);
    }

    if cli.parse_only {
        // errors should have been processed and returned above
        return true;
    }

    let (tx, iot) = spawn_listener(listener_config);

    // TODO: maybe we don't need a RunEnv
    // TODO: thread through global values? Maybe not, I have to think through this a bit more
    let run_env = RunEnv {
        emitter: Some(tx),
        ..RunEnv::default()
    };

    let mut res = true;
    for cfa in cfas {
        if !run_checks(cfa.instances, cfa.filename, &run_env) {
            res = false;
        }
    }
    iot.join().unwrap();
    res
}

// REFACTOR ME
fn huge_error_fn(filename: String, source: &String, e: ParseError<usize, Token<'_>, LexicalError>) {
    match e {
        lalrpop_util::ParseError::UnrecognizedToken { token: t, expected } => {
            let expected = expected.join(",");
            let msg = format!("Unrecognized token, expected one of: {}", expected);
            let (start, _token, end) = t;
            show_fancy_error(
                &(start..end),
                "Fatal compilation error",
                &msg,
                &source,
                &Some(filename),
            );
        }
        lalrpop_util::ParseError::InvalidToken { location: l } => {
            let loc = l..l + 1;
            let msg = "Invalid token error";
            show_fancy_error(
                &loc,
                "Fatal compilation error",
                msg,
                &source,
                &Some(filename),
            );
        }
        lalrpop_util::ParseError::UnrecognizedEof {
            location: _,
            expected: _,
        } => {
            println!("Fatal: Unrecognized EOF");
        }
        lalrpop_util::ParseError::ExtraToken { token: _ } => {
            let loc = 0..0;
            let msg = "Extra token error";
            show_fancy_error(
                &loc,
                "Fatal compilation error",
                msg,
                &source,
                &Some(filename),
            );
        }
        lalrpop_util::ParseError::User { error } => match error {
            predikit::comp::tokens::LexicalError::InvalidInteger(parse_int_error) => {
                todo!()
            }
            predikit::comp::tokens::LexicalError::InvalidType(range) => {
                show_fancy_error(
                    &(range),
                    "Invalid type",
                    "This is not a valid type",
                    &source,
                    &Some(filename),
                );
            }
            predikit::comp::tokens::LexicalError::InvalidDuration(_, range) => {
                show_fancy_error(
                    &(range),
                    "Invalid duration",
                    "This is not a valid duration literal",
                    &source,
                    &Some(filename),
                );
            }
            predikit::comp::tokens::LexicalError::InvalidPath(_, range) => {
                show_fancy_error(
                    &(range),
                    "Invalid path",
                    "This is not a valid path literal",
                    &source,
                    &Some(filename),
                );
            }
            predikit::comp::tokens::LexicalError::InvalidConversion(conv_type, msg, range) => {
                show_fancy_error(
                    &(range),
                    &format!("Invalid {}: {}", conv_type, msg),
                    "This is not a valid path literal",
                    &source,
                    &Some(filename),
                );
            }
            predikit::comp::tokens::LexicalError::InvalidToken(range) => {
                show_fancy_error(
                    &(range),
                    "Invalid token",
                    "This token is not valid",
                    &source,
                    &Some(filename),
                );
            }
        },
    }
}

// Don't bother returning a Result, the errors in this fn are fatal and predikit
// will exit after displaying these errors.
fn parse_files(cli: &Cli) -> Option<(Vec<AstFileChecks>, Vec<AstFileTools>)> {
    let mut ast_file_checks: Vec<AstFileChecks> = vec![];
    let mut ast_file_tools: Vec<AstFileTools> = vec![];
    // file DB for showing error messages. This is just a crude impl, and needs to be thought out.
    let _source_files: SimpleFiles<String, String> = SimpleFiles::new();

    // Compile each file to an ast, skipping those that fail to parse
    for infile in &cli.infiles {
        debug!("Input file = {}", infile.display());

        let source = match load_source_from_file(infile) {
            Ok(source) => source,
            Err(e) => {
                let msg = format!("error reading {}:\n{}", infile.to_string_lossy(), e);
                println!("{}", msg);
                return None;
            }
        };
        let result = pkparser::TopLevelParser::new().parse(&source);
        if let Err(e) = result {
            // it's in the name
            huge_error_fn(infile.to_string_lossy().to_string(), &source, e);
            return None;
        }

        let (checks, tools) = result.unwrap();

        // checks and tools need to carry around a filename so they can produce an
        // appropriate error message later on
        let checks = AstFileChecks::new(Some(infile.display().to_string()), checks);
        let tools = AstFileTools::new(Some(infile.display().to_string()), tools);

        // leaving this around, as it's helpful to write out this struct to a file during tracing
        let ast_file = AstFile {
            filename: Some(infile.display().to_string()),
            checks,
            tools,
        };

        let checks = ast_file.checks;
        let tools = ast_file.tools;

        ast_file_checks.push(checks);
        ast_file_tools.push(tools);
    }
    Some((ast_file_checks, ast_file_tools))
}

fn compile_checks(
    all_ast_file_checks: Vec<AstFileChecks>,
    fns: &ChkDefRegistry,
) -> Vec<CompiledCheckFileOut<'_>> {
    debug!("Compiling checks");
    crate::predikit::comp::compiler::compile_checks_to_asts(fns, all_ast_file_checks)
}

fn run_checks(root_checks: Vec<ChkInstance>, filename: Option<String>, run_env: &RunEnv) -> bool {
    // TODO: this is pretty expensive and should be rethought
    let descs: ChkDescMap = desc_from_instances(&root_checks);
    run_env.emit(ChkLifecycleEvent::Init(descs, filename.clone()));

    let mut final_result = true;
    for check in root_checks {
        let res = check.run_check_maybe_retry(run_env);
        if !res.is_check_pass() {
            // could be a fail OR an error, so just use !is_check_pass
            final_result = false;
        }
    }

    run_env.emit(ChkLifecycleEvent::Term(filename.clone()));
    final_result
}
