use crate::predikit::data::instance::{ChkDesc, ChkInstMap};
use crate::predikit::data::ChkLifecycleEvent::*;
use crate::predikit::data::{ChkInstId, ChkLifecycleEvent};
use colored::Colorize;
use log::debug;

pub struct DefaultEventListener {
    receiver: std::sync::mpsc::Receiver<ChkLifecycleEvent>,
    chk_inst_map: ChkInstMap,
}

pub fn fancy_params(cd: &ChkDesc) -> String {
}

impl DefaultEventListener {
    pub fn new(receiver: std::sync::mpsc::Receiver<ChkLifecycleEvent>) -> Self {
        Self {
            receiver,
            chk_inst_map: ChkInstMap::new(),
        }
    }

    fn find_check_by_id(&self, id: ChkInstId) -> &ChkDesc {
        self.chk_inst_map
            .iter()
            .find(|(_, chk)| chk.instance_id == id)
            .unwrap()
            .1
    }

    pub fn run(&mut self) {
        let mut depth = 0;
        fn spacer(depth: usize) {
            print!("{}", " ".repeat(depth * 4));
        }
        for event in &self.receiver {
            match event {
                Init(checks) => {
                    self.chk_inst_map = checks;
                    debug!("CHECKS: {:#?}", self.chk_inst_map);
                }

                CheckStart(inst_id) => {
                    let chk = &self.find_check_by_id(inst_id);
                    spacer(depth);
                    print!("> {} [fn={}] ", inst_id, chk.fn_name);
                    if chk.is_group {
                        println!("");
                    }
                    depth += 1;
                }
                CheckFinish(inst_id, duration) => {
                    let chk = &self.find_check_by_id(inst_id);
                    println!(" [{}μs]", duration.as_micros());
                }

                CheckFail(inst_id) => {
                    let chk = &self.find_check_by_id(inst_id);
                    depth -= 1;
                    if chk.is_group {
                        spacer(depth);
                    }
                    print!("{} {}", ">", "Fail".yellow());
                }
                CheckPass(inst_id) => {
                    let chk = &self.find_check_by_id(inst_id);
                    depth -= 1;
                    if chk.is_group {
                        spacer(depth);
                    }
                    print!("{} {}", ">", "Pass".green());
                }
                CheckError(inst_id) => {
                    let chk = &self.find_check_by_id(inst_id);
                    depth -= 1;
                    if chk.is_group {
                        spacer(depth);
                    }
                    print!("{} {}", ">", "Error".red());
                }
                Term => {
                    break;
                }
            }
        }
        println!("Done");
    }
}


