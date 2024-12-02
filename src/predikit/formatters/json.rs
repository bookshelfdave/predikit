// Copyright (c) 2025 Dave Parfitt

use crate::predikit::data::events::ChkLifecycleEvent::*;
use crate::predikit::data::events::{ChkDesc, ChkDescMap, ChkDescResult, ChkLifecycleEvent};
use crate::predikit::data::instance::ChkInstId;
use colored::Colorize;
use log::debug;
use std::io::stdout;
use std::io::Write;

use super::{FormatterConfig, OutputFormatter};

enum PathType {
    Group,
    Check,
}

pub struct JsonOutputFormatter {
    config: FormatterConfig,
    chk_inst_map: ChkDescMap,
}

impl Default for JsonOutputFormatter {
    fn default() -> Self {
        Self {
            config: FormatterConfig::default(),
            chk_inst_map: ChkDescMap::new(),
        }
    }
}

fn show_tree(path_stack: &Vec<PathType>) {
    path_stack.iter().for_each(|segment| match segment {
        PathType::Check => print!("->"),
        PathType::Group => print!("|  "),
    });
}

impl OutputFormatter for JsonOutputFormatter {
    fn init(&mut self, cfg: FormatterConfig) {
        self.config = cfg;
    }

    fn process_events(&mut self, receiver: std::sync::mpsc::Receiver<ChkLifecycleEvent>) {
        self.process_loop(receiver);
    }

    fn term(&mut self) {}
}

impl JsonOutputFormatter {
    pub fn new(config: FormatterConfig) -> Self {
        Self {
            config,
            chk_inst_map: ChkDescMap::new(),
        }
    }

    fn find_check_by_id_mut(&mut self, id: ChkInstId) -> &mut ChkDesc {
        self.chk_inst_map
            .iter_mut()
            .find(|(_, chk)| chk.instance_id == id)
            .unwrap()
            .1
    }

    fn find_check_by_id(&self, id: ChkInstId) -> &ChkDesc {
        self.chk_inst_map
            .iter()
            .find(|(_, chk)| chk.instance_id == id)
            .unwrap()
            .1
    }

    fn get_root_checks(&self) -> Vec<&ChkDesc> {
        self.chk_inst_map
            .iter()
            .filter(|(_, chk)| chk.is_root)
            .map(|(_, chk)| chk)
            .collect()
    }

    fn fancy_params(&self, chk: &ChkDesc) -> String {
        chk.fn_desc
            .formal_params
            .iter()
            .map(|param| {
                let param_name = param.0.clone();
                let param_value = chk.actual_params.get(&param_name).unwrap();
                format!("{}: {}", param_name, param_value.value_as_string())
            })
            .collect::<Vec<String>>()
            .join(", ")
    }

    pub fn process_events(&mut self, receiver: std::sync::mpsc::Receiver<ChkLifecycleEvent>) {
        self.process_loop(receiver);
        self.finish_processing_events();
    }

    fn finish_processing_events(&mut self) {
        let root_checks = self.get_root_checks();
        debug!("{:#?}", root_checks);
        let all_pass = root_checks
            .iter()
            .all(|chk| matches!(chk.result, Some(ChkDescResult::Pass)));
        if all_pass {
            println!("{}", "All root checks passed".truecolor(0, 200, 0));
        } else {
            let some_pass = root_checks
                .iter()
                .any(|chk| matches!(chk.result, Some(ChkDescResult::Pass)));
            if some_pass {
                println!("{}", "Some root checks failed".truecolor(200, 0, 0));
            } else {
                println!("{}", "All root checks failed".truecolor(200, 0, 0));
            }
        }
    }

    fn get_title(&self, chk: &ChkDesc) -> String {
        let p = chk.actual_params.get("title");
        if let Some(p) = p {
            format!("[{}] ", p.get_string())
        } else {
            "".to_string()
        }
    }

    fn process_loop(&mut self, receiver: std::sync::mpsc::Receiver<ChkLifecycleEvent>) {
        let mut path_stack: Vec<PathType> = vec![];

        for event in receiver.iter() {
            match event {
                Init(checks, filename) => {
                    println!(
                        "\n* Running tests from {}:",
                        filename.unwrap_or("<no file>".to_string())
                    );
                    self.chk_inst_map = checks;
                    debug!("CHECKS: {:#?}", self.chk_inst_map);
                }

                CheckStart(inst_id) => {
                    let chk = self.find_check_by_id(inst_id);
                    let title = self.get_title(chk).blue();
                    let negate = if chk.negated { " not " } else { "" };

                    if chk.is_group {
                        path_stack.push(PathType::Group);
                    } else {
                        path_stack.push(PathType::Check);
                    }
                    show_tree(&path_stack);
                    print!(
                        " {}{}[{}] ",
                        title,
                        negate.purple(),
                        chk.fn_desc.fn_name.blue()
                    );
                    print!("{}", self.fancy_params(chk));
                    if chk.is_group {
                        println!();
                    }
                }
                CheckFinish(inst_id, duration) => {
                    let _chk = &self.find_check_by_id_mut(inst_id);
                    println!(" [{}μs]", duration.as_micros());
                    let _ = path_stack.pop();
                }
                CheckRetrySleep(_inst_id, seconds) => {
                    let msg = format!("  Sleep {} seconds...", seconds);
                    print!("  {}", msg.bright_yellow());
                    let _ = stdout().flush();
                }
                CheckRetry(_inst_id, attempt) => {
                    let msg = format!("  Retry {}", attempt);
                    println!("  {}", msg.bright_yellow());
                }
                CheckFail(inst_id) => {
                    let chk = &self.find_check_by_id_mut(inst_id);
                    if chk.is_group {
                        show_tree(&path_stack);
                    }
                    print!(" {}", "Fail".truecolor(200, 0, 0).bold());
                }
                CheckPass(inst_id) => {
                    let chk = &mut self.find_check_by_id_mut(inst_id);
                    chk.update_result(ChkDescResult::Pass);
                    if chk.is_group {
                        show_tree(&path_stack);
                    }
                    print!(" {}", "Pass".truecolor(00, 200, 0).bold());
                }
                CheckError(inst_id) => {
                    let chk = &self.find_check_by_id_mut(inst_id);
                    if chk.is_group {
                        show_tree(&path_stack);
                    }
                    print!(" {}", "Error".red());
                }
                Term(filename) => {
                    println!(
                        "* Finished running tests from {}",
                        filename.unwrap_or("<no file>".to_string())
                    );
                    break;
                }
            }
        }
    }
}
