pub mod instance;

use crate::predikit::data::instance::ChkInstMap;
use crate::predikit::functions::builtin;
use instance::ChkInstance;
use std::collections::HashMap;
use std::fmt;

type ChkFunctionParams = HashMap<String, ChkParamInstance>;
type ChkFn = fn(
    &RunEnv,
    &ChkFunctionParams,
    &ChkInstance,
) -> ChkResult;

#[derive(Debug, Clone)]
pub struct ChkDef {
    pub name: String,
    pub check_fn: ChkFn,
    pub formal_params: Vec<ChkFormalParam>,
    pub accepts_children: bool,
    pub is_group: bool,
}
#[derive(Debug, Clone)]
pub enum ChkParamType {
    PkString,
    PkInt,
    PkBool,
}

impl fmt::Display for ChkParamType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub enum ChkParamInstance {
    PkString(String),
    PkInt(i64),
    PkBool(bool),
}

impl ChkParamInstance {
    pub fn get_string(&self) -> String {
        if let ChkParamInstance::PkString(s) = &self {
            s.to_owned()
        } else {
            panic!("Expected a string");
        }
    }

    pub fn get_int(&self) -> i64 {
        if let ChkParamInstance::PkInt(i) = &self {
            *i
        } else {
            panic!("Expected an int");
        }
    }

    pub fn get_bool(&self) -> bool {
        if let ChkParamInstance::PkBool(b) = &self {
            *b
        } else {
            panic!("Expected a bool");
        }
    }
}
impl fmt::Display for ChkParamInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type ChkActualParams = HashMap<String, ChkParamInstance>;

#[derive(Debug, Clone)]
pub struct ChkFormalParam {
    pub name: String,
    pub required: bool,
    pub param_type: ChkParamType,
    pub param_default: Option<ChkParamInstance>,
}

impl fmt::Display for ChkFormalParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let def = if let Some(d) = &self.param_default {
            format!(" (default: {})", d)
        } else {
            "".to_owned()
        };

        write!(
            f,
            "{} ({}) [{}={}]",
            self.name,
            if self.required { "required" } else { "optional" },
            self.param_type.to_string(),
            def,
        )
    }
}

pub struct FParamBuilder {
    name: String,
    required: bool,
    param_type: ChkParamType,
    param_default: Option<ChkParamInstance>,
}

impl FParamBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required: true,
            param_type: ChkParamType::PkString,
            param_default: None,
        }
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn not_required(mut self) -> Self {
        self.required = false;
        self
    }

    pub fn pk_int(mut self) -> Self {
        self.param_type = ChkParamType::PkInt;
        self
    }

    pub fn pk_bool(mut self) -> Self {
        self.param_type = ChkParamType::PkBool;
        self
    }

    pub fn pk_string(mut self) -> Self {
        self.param_type = ChkParamType::PkString;
        self
    }

    pub fn param_type(mut self, param_type: ChkParamType) -> Self {
        self.param_type = param_type;
        self
    }

    pub fn default_value(mut self, default: ChkParamInstance) -> Self {
        self.param_default = Some(default);
        self
    }

    pub fn build(self) -> ChkFormalParam {
        ChkFormalParam {
            name: self.name,
            required: self.required,
            param_type: self.param_type,
            param_default: self.param_default,
        }
    }
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

pub struct FinishedCheck<'a> {
    pub instance: ChkInstance<'a>,
    pub result: ChkResult,
}

impl<'a> FinishedCheck<'a> {
    pub fn new(instance: ChkInstance<'a>, result: ChkResult) -> Self {
        Self {
            instance,
            result,
        }
    }
}

// impl fmt::Display for ChkResult {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match &self.result {
//             Ok(bool_result) => writeln!(f, "Check result: {}", bool_result),
//             Err(err_msg) => writeln!(f, "Check error: {}", err_msg),
//         }?;
//
//
//         Ok(())
//     }
// }


pub type ChkInstId = usize;

#[derive(Debug)]
pub enum ChkLifecycleEvent {
    Init(ChkInstMap),
    Term,
    //AllStart,
    //AllFinish,
    CheckStart(ChkInstId),
    CheckPass(ChkInstId),
    CheckFail(ChkInstId),
    CheckError(ChkInstId),
    CheckFinish(ChkInstId, std::time::Duration),
}

#[derive(Default)]
pub struct RunEnv {
    //pub verbosity: CheckVerbosity,
    pub check_depth: u8,

    pub emitter: Option<std::sync::mpsc::Sender<ChkLifecycleEvent>>,
}

impl RunEnv {
    pub fn emit(&self, event: ChkLifecycleEvent) {
        self.emitter.as_ref().unwrap().send(event).unwrap();
    }

    pub fn new_check_scope(&self, inst_id: ChkInstId) -> ChkEventScope {
        ChkEventScope::new(self, inst_id)
    }
}

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
                self.run_env.emit(ChkLifecycleEvent::CheckError(self.chk_id));
            }
        }
    }
}


impl<'a> Drop for ChkEventScope<'a> {
    fn drop(&mut self) {
        self.check_duration = Some(self.start_time.elapsed());
        self.run_env.emit(ChkLifecycleEvent::CheckFinish(self.chk_id, self.check_duration.unwrap()));
    }
}


#[derive(Debug, Default)]
pub struct ChkDefRegistry {
    checks: HashMap<String, ChkDef>,
    aggs: HashMap<String, ChkDef>,
}

impl ChkDefRegistry {
    pub fn new_with_builtins() -> ChkDefRegistry {
        let mut reg = ChkDefRegistry::default();
        for check_def in builtin::define_builtins() {
            reg.register_check(check_def.name.clone(), check_def);
        }
        for agg_def in builtin::define_aggs() {
            reg.register_agg(agg_def.name.clone(), agg_def);
        }

        reg
    }
    pub fn register_check(&mut self, name: String, check_def: ChkDef) {
        self.checks.insert(name.clone(), check_def);
    }

    pub fn get_check(&self, name: &str) -> Option<&ChkDef> {
        self.checks.get(name)
    }


    pub fn register_agg(&mut self, name: String, check_def: ChkDef) {
        self.aggs.insert(name.clone(), check_def);
    }

    pub fn get_agg(&self, name: &str) -> Option<&ChkDef> {
        self.aggs.get(name)
    }
}