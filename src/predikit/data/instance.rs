// Copyright (c) 2025 Dave Parfitt

use crate::predikit::data::events::{ChkEventScope, ChkLifecycleEvent};
use crate::predikit::data::params::ChkActualParam;
use crate::predikit::data::{ChkDef, ChkFormalParam};
use log::debug;
use std::collections::HashMap;
use std::{fmt, thread};

use super::params::ChkActualParams;
use super::{ChkFormalParams, FParamBuilder, ParsedDuration};

pub type ContentAddress = std::ops::Range<usize>;

pub struct ChkInstancePreMaterialized<'a> {
    pub title: Option<String>,
    pub fn_def: Option<&'a ChkDef>,
    pub actual_params: ChkActualParams,
    pub materialized_formal_params: Option<ChkFormalParams>,
    pub children: Vec<ChkInstancePreMaterialized<'a>>,
    pub negated: bool,
    pub is_retrying: bool,
    pub instance_id: usize,
    pub content_address: ContentAddress,
    pub is_query: bool,
}

#[derive(Clone, Debug)]
pub struct ChkInstance<'chkdef> {
    pub title: Option<String>,
    pub fn_def: &'chkdef ChkDef,
    pub actual_params: ChkActualParams,
    pub materialized_formal_params: Option<ChkFormalParams>,
    pub children: Vec<ChkInstance<'chkdef>>,
    pub negated: bool,
    pub is_retrying: bool,
    pub instance_id: usize,
    pub content_address: ContentAddress,
    pub is_query: bool,
}

impl<'a> From<ChkInstancePreMaterialized<'a>> for ChkInstance<'a> {
    fn from(inst2: ChkInstancePreMaterialized<'a>) -> Self {
        ChkInstance {
            title: inst2.title,
            fn_def: inst2.fn_def.unwrap(),
            actual_params: inst2.actual_params,
            materialized_formal_params: inst2.materialized_formal_params,
            children: inst2.children.into_iter().map(|c| c.into()).collect(),
            negated: inst2.negated,
            is_retrying: inst2.is_retrying,
            instance_id: inst2.instance_id,
            content_address: inst2.content_address,
            is_query: inst2.is_query,
        }
    }
}

impl fmt::Display for ChkInstance<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Show check name and negation status
        writeln!(
            f,
            "{}{}",
            if self.negated { "not " } else { "" },
            self.fn_def.name
        )?;

        Ok(())
    }
}

const TITLE: &str = "title";
const HOOK_ON_PASS: &str = "on_pass";
const HOOK_ON_FAIL: &str = "on_fail";
const HOOK_ON_ERROR: &str = "on_error";
const HOOK_ON_INIT: &str = "on_init";
const HOOK_ON_TERM: &str = "on_term";

const META_PARAMS: [&str; 6] = [
    TITLE,
    HOOK_ON_PASS,
    HOOK_ON_FAIL,
    HOOK_ON_ERROR,
    HOOK_ON_INIT,
    HOOK_ON_TERM,
];

impl ChkInstance<'_> {
    pub fn materialize_formal_params(&mut self) {
        let mut fps = self.fn_def.formal_params.clone();
        fn build_param(name: &str) -> ChkFormalParam {
            FParamBuilder::new(name).pk_string().not_required().build()
        }

        for mp in META_PARAMS {
            fps.insert(mp.to_owned(), build_param(mp));
        }

        // add retrying paramaters
        if self.is_retrying {
            // TODO: default param values!
            let retries = FParamBuilder::new("retries").pk_int().required().build();
            fps.insert("retries".to_owned(), retries);

            let retry_delay = FParamBuilder::new("retry_delay")
                .pk_duration()
                .required()
                .build();
            fps.insert("retry_delay".to_owned(), retry_delay);
        }
        self.materialized_formal_params = Some(fps);
    }

    fn run_hook_if_defined(&self, hook_name: &str) {
        if let Some(hook_value) = self.actual_params.get(hook_name) {
            let hook_value = hook_value.get_string();
            //println!("Running check {}->[{}]", hook_name, hook_value);
            let cmd_result = std::process::Command::new("sh")
                .arg("-c")
                .arg(hook_value)
                .output();
            if let Ok(out) = cmd_result {
                // TODO: address unwraps
                println!(
                    "[{}]: {}",
                    hook_name,
                    String::from_utf8(out.stdout).unwrap()
                );
                // TODO: what to do about stdout?
                //   it would be cool to be able to set all on_* shell params globalls
                //    ex: set the shell for all on_* hooks to /bin/bash -c
                //
                // println!(
                //     "[{}]: {}",
                //     hook_name,
                //     String::from_utf8(out.stderr).unwrap()
                // );
            } else {
                // TODO: what to do here?
                println!("Error running {} hook command", hook_name);
            }
        } else {
            debug!(
                "Hook {} is not defined for this check {}",
                hook_name, self.fn_def.name
            );
        }
    }

    fn sleep_and_emit_retry_events(
        &self,
        run_env: &RunEnv,
        retry_delay: &ParsedDuration,
        attempt_num: u64,
    ) {
        let duration = retry_delay.duration;
        run_env.emit(ChkLifecycleEvent::CheckRetrySleep(
            self.instance_id,
            retry_delay.clone(),
        ));

        thread::sleep(duration);
        run_env.emit(ChkLifecycleEvent::CheckRetry(self.instance_id, attempt_num));
    }

    fn eval_result_for_hook(&self, r: &ChkResult) {
        //println!("EVAL HOOK: {:#?}", r);
        if r.is_check_pass() {
            self.run_hook_if_defined(HOOK_ON_PASS);
        } else if r.is_check_fail() {
            self.run_hook_if_defined(HOOK_ON_FAIL);
        } else if r.is_check_error() {
            self.run_hook_if_defined(HOOK_ON_ERROR);
        } else {
            panic!("Unknown check state");
        }
    }

    fn run_check_no_retry(&self, run_env: &RunEnv) -> ChkResult {
        self.run_hook_if_defined(HOOK_ON_INIT);
        let r = self.exec(run_env);
        self.eval_result_for_hook(&r);
        self.run_hook_if_defined(HOOK_ON_TERM);
        r
    }

    pub fn run_check_maybe_retry(&self, run_env: &RunEnv) -> ChkResult {
        if !self.is_retrying {
            return self.run_check_no_retry(run_env);
        }

        // retries doesn't have a default, so it's safe to unwrap
        // as the compiler will enforce that retries is required.
        // at least for now!
        // TODO: won't this fail if someone tries a negative retries value?
        let retries = self.actual_params.get("retries").unwrap().get_int() as u64;
        // TODO: should probably not convert to u64 and just support 32 bit numbers for retries..?
        // TODO: won't this fail if someone tries a negative retry_delay?
        let retry_delay = self
            .actual_params
            .get("retry_delay")
            .unwrap()
            .get_duration();

        self.run_hook_if_defined(HOOK_ON_INIT);

        // I need the last value returned, which is annoying in rust
        for attempt_num in 1..=retries {
            let attempt_result = self.exec(run_env);

            if self.is_query {
                // possibly modify the attempt_result here
                // query todo:
                // expressions ( > 1 && < 5 ) ? not sure if it's worth it
                println!("Process is_query predicate");
            }

            self.eval_result_for_hook(&attempt_result);
            if attempt_num == retries || attempt_result.is_check_pass() {
                self.eval_result_for_hook(&attempt_result);
                self.run_hook_if_defined(HOOK_ON_TERM);
                return attempt_result;
            } else {
                self.sleep_and_emit_retry_events(run_env, &retry_delay, attempt_num);
                continue;
            }
        }
        unreachable!();
    }

    fn exec(&self, run_env: &RunEnv) -> ChkResult {
        let chk_scope = run_env.new_check_scope(self.instance_id);
        let check_run = (self.fn_def.check_fn)(run_env, &self.actual_params, self);
        if self.negated {
            debug!("Negating result");
            let final_val = !check_run.result.unwrap();
            ChkResult {
                result: Ok(final_val),
                process_out: check_run.process_out,
                children_results: check_run.children_results,
            }
        } else {
            chk_scope.emit_result(&check_run);
            check_run
        }
    }
}

pub struct ChkInstanceBuilder<'a> {
    title: Option<String>,
    check_def: &'a ChkDef,
    actual_params: ChkActualParams,
    children: Vec<ChkInstance<'a>>,
    negated: bool,
    is_retrying: bool,
    instance_id: usize,
    is_query: bool,
}

impl<'a> ChkInstanceBuilder<'a> {
    pub fn new(check_def: &'a ChkDef) -> Self {
        Self {
            title: None,
            check_def,
            actual_params: HashMap::new(),
            children: Vec::new(),
            negated: false,
            is_retrying: false,
            instance_id: 1000,
            is_query: false,
        }
    }

    pub fn title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    pub fn param(mut self, name: impl Into<String>, value: ChkActualParam) -> Self {
        self.actual_params.insert(name.into(), value);
        self
    }

    pub fn param_string(mut self, name: impl Into<String>, value: &str) -> Self {
        let name: String = name.into();
        self.actual_params.insert(
            name.clone(),
            ChkActualParam::new_string(
                name,
                value.to_owned(),
                std::ops::Range { start: 0, end: 0 },
            ),
        );
        self
    }

    pub fn param_int(mut self, name: impl Into<String>, value: i64) -> Self {
        let name: String = name.into();
        self.actual_params.insert(
            name.clone(),
            ChkActualParam::new_int(name, value, std::ops::Range { start: 0, end: 0 }),
        );
        self
    }

    pub fn param_bool(mut self, name: impl Into<String>, value: bool) -> Self {
        let name: String = name.into();
        self.actual_params.insert(
            name.clone(),
            ChkActualParam::new_bool(name, value, std::ops::Range { start: 0, end: 0 }),
        );
        self
    }

    pub fn with_params(mut self, params: ChkActualParams) -> Self {
        self.actual_params.extend(params);
        self
    }

    pub fn add_child(mut self, child: ChkInstance<'a>) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<ChkInstance<'a>>) -> Self {
        self.children = children;
        self
    }

    pub fn instance_id(mut self, instance_id: usize) -> Self {
        self.instance_id = instance_id;
        self
    }

    pub fn negated(mut self, val: bool) -> Self {
        self.negated = val;
        self
    }

    pub fn retrying(mut self) -> Self {
        self.is_retrying = true;
        self
    }

    pub fn query(mut self) -> Self {
        self.is_query = true;
        self
    }

    pub fn build(self) -> ChkInstance<'a> {
        ChkInstance {
            title: self.title,
            fn_def: self.check_def,
            actual_params: self.actual_params,
            materialized_formal_params: None,
            children: self.children,
            negated: self.negated,
            is_retrying: self.is_retrying,
            instance_id: self.instance_id,
            content_address: ContentAddress::default(), // TODO: add position info to the builder
            is_query: self.is_query,
        }
    }
}

pub type ChkInstId = usize;

#[cfg(test)]
mod tests {
    #[test]
    fn test_chk_param_instance_force_to_string() {}
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ChkProcessOut {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: Option<i32>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChkResult {
    pub result: Result<bool, String>,
    pub process_out: Option<ChkProcessOut>,
    pub children_results: Option<Vec<ChkResult>>,
}

impl ChkResult {
    pub fn is_check_pass(&self) -> bool {
        if let Ok(r) = self.result.as_ref() {
            *r
        } else {
            false
        }
    }

    pub fn is_check_fail(&self) -> bool {
        if let Ok(r) = self.result {
            !r
        } else {
            false
        }
    }

    pub fn is_check_error(&self) -> bool {
        self.result.is_err()
    }
}

#[derive(Default)]
pub struct RunEnv {
    // used to send check events to a formatter, which generates (console or other) output
    pub emitter: Option<std::sync::mpsc::Sender<ChkLifecycleEvent>>,
    pub global_config: ChkActualParams,
}

impl RunEnv {
    pub fn emit(&self, event: ChkLifecycleEvent) {
        self.emitter.as_ref().unwrap().send(event).unwrap();
    }

    pub fn new_check_scope(&self, inst_id: ChkInstId) -> ChkEventScope {
        ChkEventScope::new(self, inst_id)
    }
}
