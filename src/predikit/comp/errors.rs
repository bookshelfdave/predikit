// Copyright (c) 2025 Dave Parfitt

use std::collections::HashMap;

use crate::predikit::data::instance::ContentAddress;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::DisplayStyle;

use super::CompileError;

pub fn show_fancy_error(
    ca: &ContentAddress,
    title: &str,
    description: &str,
    source: &String,
    filename: &Option<String>,
) {
    let mut files = SimpleFiles::new();
    let fname = match filename {
        None => "".to_string(),
        Some(f) => f.clone(),
    };
    let file_id = files.add(fname, &source);
    let diag = Diagnostic::error()
        .with_message(title)
        .with_labels(vec![
            Label::primary(file_id, ca.clone()).with_message(description)
        ]);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut config = codespan_reporting::term::Config::default();
    config.display_style = DisplayStyle::Rich;

    codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diag).unwrap();
}

pub fn show_fancy_compile_errors(ces: Vec<CompileError>) {
    // so many clones!
    let file_names: Vec<String> = ces
        .iter()
        .map(|ce| ce.filename.as_ref().unwrap_or(&"".to_string()).clone())
        .collect();

    let mut files = SimpleFiles::new();

    let mut file_ids: HashMap<String, usize> = HashMap::new();
    for fname in file_names {
        let source = std::fs::read_to_string(&fname).expect("Error reading file");
        let file_id = files.add(fname.clone(), source);
        // Note: if there are multiple empty filenames, the file_id is incremented for each.
        // I need to think about it more, but maybe it's not an issue?
        file_ids.insert(fname, file_id);
    }
    for ce in ces {
        let fname = ce.filename.as_ref().unwrap_or(&"".to_string()).clone();
        let file_id = file_ids.get(&fname).unwrap();
        let diag = Diagnostic::error()
            .with_message("Compile error".to_string())
            .with_labels(vec![
                Label::primary(*file_id, ce.content_address.clone()).with_message(ce.message)
            ]);

        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();

        codespan_reporting::term::emit(&mut writer.lock(), &config, &files, &diag).unwrap();
    }
}
