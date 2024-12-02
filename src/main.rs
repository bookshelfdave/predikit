use crate::predikit::comp::{file_to_ir, FuncReg, IRMapCtx};
use crate::predikit::data::instance::{desc_from_insts, ChkInstMap};
use crate::predikit::data::{ChkLifecycleEvent, FinishedCheck, RunEnv};
use crate::predikit::formatter::DefaultEventListener;
use clap::Parser;
use log::debug;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

// made this pub to get rid of the unused warnings
pub mod predikit;
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    name: Option<String>,

    #[arg(short, long, value_name = "FILE")]
    infile: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Predikit 0.1.0");

    let cli = Cli::parse();
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    let log_level = match cli.debug {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Info,
        _ => log::LevelFilter::Debug,
    };
    env_logger::builder()
        .filter_level(log_level)
        .init();
    debug!("Log level: {}", log_level);

    let (tx, rx) = channel();

    let iot = thread::spawn(move || {
        let mut listener = DefaultEventListener::new(rx);
        listener.run();
        // for received in rx {
        //     println!("IO thread: {:#?}", received);
        //     if let ChkLifecycleEvent::Term = received {
        //         println!("IO Term received");
        //         return;
        //     }
        // }
    });

    let mut re = RunEnv::default();
    re.emitter = Some(tx);

    // THIS ALL NEEDS TO MOVED!
    if let Some(infile) = cli.infile.as_deref() {
        debug!("Input file = {}", infile.display());
        let ast = crate::predikit::comp::compile_file(infile)?;
        let funs = FuncReg::new();
        let mut ctx = IRMapCtx::new();
        let ir = file_to_ir(&funs, &mut ctx, ast);
        if ir.is_err() {
            panic!("Error parsing file: {}", infile.display());
        }
        debug!("IR: {:#?}", ir);

        let file_ir = ir.unwrap();

        //let descs: Vec<ChkDesc> = file_ir.iter().map(|ci| ChkDesc::from(ci)).collect();

        let descs: ChkInstMap = desc_from_insts(&file_ir);

        re.emit(ChkLifecycleEvent::Init(descs));

        let _file_results: Vec<FinishedCheck> =
            file_ir
                .into_iter()
                .map(|ci| {
                    let cr = ci.run_check(&re);
                    FinishedCheck::new(ci, cr)
                })
                .collect();

        // debug!("File results: {:#?}", file_results);
        // for ci in file_ir {
        //     let result = ci.run_check(&re);
        //     FinishedCheck::new(ci, result);
        //     println!("Check instance {}", ci);
        // }
        // // TODO: return a Result
    }

    re.emit(ChkLifecycleEvent::Term);
    iot.join().unwrap();
    Ok(())
}