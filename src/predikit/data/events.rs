// Copyright (c) 2025 Dave Parfitt

use crate::predikit::data::instance::{ChkInstId, ChkInstance};
use crate::predikit::data::params::ChkActualParam;
use crate::predikit::data::{ChkFormalParam, ChkResult, RunEnv};
use std::collections::HashMap;

use super::ParsedDuration;

/// Events that are sent to the events layer via an event emitter.
#[derive(Debug)]
pub enum ChkLifecycleEvent {
    Init(ChkDescMap, Option<String>), // filename // TODO: this is super expensive for large amounts of checks
    Term(Option<String>),             // filename
    //AllStart,
    //AllFinish,
    CheckRetry(ChkInstId, u64),                 // attempt #
    CheckRetrySleep(ChkInstId, ParsedDuration), // sleep in seconds
    CheckStart(ChkInstId),
    CheckPass(ChkInstId),
    CheckFail(ChkInstId),
    CheckError(ChkInstId),
    CheckFinish(ChkInstId, std::time::Duration),
}

/// A scope used for timing a check. Emits Pass, Fail, or Error events to the emitter.
/// When ChkEventScope is dropped, the duration is sent to the event emitter as a CheckFinish event.
pub struct ChkEventScope<'a> {
    run_env: &'a RunEnv,
    chk_id: ChkInstId,
    start_time: std::time::Instant,
    check_duration: Option<std::time::Duration>,
}

impl<'a> ChkEventScope<'a> {
    // implement new
    pub fn new(run_env: &'a RunEnv, chk_id: ChkInstId) -> Self {
        run_env.emit(ChkLifecycleEvent::CheckStart(chk_id));
        Self {
            run_env,
            chk_id,
            start_time: std::time::Instant::now(),
            check_duration: None,
        }
    }

    pub fn emit_result(&self, r: &ChkResult) {
        // emit an error event if r is an error, otherwise emit pass or fail
        // depending on the result
        match &r.result {
            Ok(bool_result) => {
                if *bool_result {
                    self.run_env.emit(ChkLifecycleEvent::CheckPass(self.chk_id));
                } else {
                    self.run_env.emit(ChkLifecycleEvent::CheckFail(self.chk_id));
                }
            }
            Err(_) => {
                self.run_env
                    .emit(ChkLifecycleEvent::CheckError(self.chk_id));
            }
        }
    }
}

impl Drop for ChkEventScope<'_> {
    fn drop(&mut self) {
        self.check_duration = Some(self.start_time.elapsed());
        self.run_env.emit(ChkLifecycleEvent::CheckFinish(
            self.chk_id,
            self.check_duration.unwrap(),
        ));
    }
}

/// A description of a check instance combined with its definition and parameters, suitable
/// for passing to the events layer for output (console or otherwise).
#[derive(Debug)]
pub struct ChkDesc {
    pub instance_id: usize,
    pub fn_desc: ChkFnDesc,
    pub actual_params: HashMap<String, ChkActualParam>,
    //pub formal_params: HashMap<String, ChkFormalParam>,
    // pass functions as a separate thing
    pub children: Vec<ChkInstId>,
    pub negated: bool,
    pub is_group: bool,
    pub is_retrying: bool,
    pub title: Option<String>,
    pub is_root: bool, // is this check at the top level of a check file?
    pub result: Option<ChkDescResult>,
}

impl ChkDesc {
    pub fn update_result(&mut self, new_result: ChkDescResult) {
        self.result = Some(new_result);
    }
}

#[derive(Debug)]
pub enum ChkDescResult {
    Pass,
    Fail,
    Error,
}

/// A description of a check function, including its name and formal parameters.
#[derive(Debug)]
pub struct ChkFnDesc {
    pub fn_name: String,
    pub formal_params: HashMap<String, ChkFormalParam>,
}

/// A map of check instance ids to their descriptions. This is used in the events layer.
pub type ChkDescMap = HashMap<ChkInstId, ChkDesc>;

/// Recursively create a map of check instance ids to their descriptions from a vector of check instances.
pub fn desc_from_instances(instances: &Vec<ChkInstance>) -> ChkDescMap {
    fn _desc_from_inst(i: &ChkInstance, v: &mut ChkDescMap, is_root: bool) {
        let child_ids: Vec<ChkInstId> = i.children.iter().map(|c| c.instance_id).collect();
        for child in &i.children {
            _desc_from_inst(child, v, false);
        }

        let mut formal_params = HashMap::new();
        for p in i.fn_def.formal_params.values() {
            formal_params.insert(p.name.clone(), p.clone());
        }

        let this = ChkDesc {
            fn_desc: ChkFnDesc {
                fn_name: i.fn_def.name.clone(),
                formal_params,
            },
            actual_params: i.actual_params.clone(),
            children: child_ids,
            negated: i.negated,
            is_group: i.fn_def.is_group,
            is_retrying: i.is_retrying,
            instance_id: i.instance_id,
            title: i.title.clone(),
            is_root,
            result: None,
        };
        v.insert(i.instance_id, this);
    }

    let mut v: ChkDescMap = HashMap::new();
    for i in instances {
        _desc_from_inst(i, &mut v, true);
    }

    v
}
