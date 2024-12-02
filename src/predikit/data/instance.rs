use crate::predikit::data::{ChkActualParams, ChkDef, ChkFormalParam, ChkInstId, ChkParamInstance, ChkResult, RunEnv};
use log::debug;
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug)]
pub struct ChkInstance<'a> {
    pub title: Option<String>,
    pub check_def: &'a ChkDef,
    pub actual_params: ChkActualParams,
    pub children: Vec<ChkInstance<'a>>,
    pub negated: bool,
    pub is_retrying: bool,
    pub instance_id: usize,
}

#[derive(Debug)]
pub struct ChkDesc {
    pub fn_name: String,
    pub actual_params: HashMap<String, ChkParamInstance>,
    //pub formal_params: HashMap<String, ChkFormalParam>,
    // pass functions as a separate thing
    pub children: Vec<ChkInstId>,
    pub negated: bool,
    pub is_group: bool,
    pub is_retrying: bool,
    pub instance_id: usize,
    pub title: Option<String>,
}

#[derive(Debug)]
pub struct ChkFnDesc {
    pub fn_name: String,
    pub formal_params: HashMap<String, ChkFormalParam>,
}

pub type ChkInstMap = HashMap<ChkInstId, ChkDesc>;

pub fn desc_from_insts(instances: &Vec<ChkInstance>) -> ChkInstMap {
    fn _desc_from_inst(i: &ChkInstance, v: &mut ChkInstMap) {
        let child_ids: Vec<ChkInstId> = i.children.iter().map(|c| c.instance_id).collect();
        for child in &i.children {
            _desc_from_inst(child, v);
        }
        let this = ChkDesc {
            fn_name: i.check_def.name.clone(),
            actual_params: i.actual_params.clone(),
            children: child_ids,
            negated: i.negated,
            is_group: i.check_def.is_group,
            is_retrying: i.is_retrying,
            instance_id: i.instance_id,
            title: i.title.clone(),
        };
        v.insert(i.instance_id, this);
    }

    let mut v: ChkInstMap = HashMap::new();
    for i in instances {
        _desc_from_inst(i, &mut v);
    }

    v
}

impl<'a> fmt::Display for ChkInstance<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        // Show check name and negation status
        writeln!(f, "{}{}",
                 if self.negated { "not " } else { "" },
                 self.check_def.name
        )?;

        Ok(())
    }
}

impl<'a> ChkInstance<'a> {
    fn verify_params(&self) -> Result<(), ChkResult> {
        // TODO:
        // 1. Check that all required parameters are present
        // 2. Check that no extra parameters were provided
        // 3. Check that the types of the parameters are correct
        // 4. Check that the default values are correct?
        // 5. Check that the default values are not used if the parameter is required?

        let _required_params_count = self
            .check_def
            .formal_params
            .iter()
            .filter(|param| param.required)
            .count();


        // Then validate no extra parameters were provided

        self.check_def.formal_params.iter().for_each(|param| {
            if !self.actual_params.contains_key(&param.name) {
                let msg = format!(
                    "Missing parameter {} for function {}",
                    param.name, self.check_def.name
                );
                panic!("{}", msg);
            }
        });


        // look in actual_params for any parameters that are not in formal_params

        Ok(())
    }

    pub fn run_check(&self, run_env: &RunEnv) -> ChkResult {
        //print!("[{}] {}", self.title.as_deref().unwrap_or("-"), self.check_def.name);
        let chk_scope = run_env.new_check_scope(self.instance_id);
        let r = match self.verify_params() {
            Err(check_err) => check_err,
            Ok(_) => {
                let r = (self.check_def.check_fn)(run_env, &self.actual_params, self);
                if r.result.is_err() {
                    chk_scope.emit_result(&r);
                    r
                } else if self.negated {
                    debug!("Negating result");
                    let final_val = !r.result.unwrap();
                    //println!("{}", final_val);
                    ChkResult {
                        result: Ok(final_val),
                        process_out: r.process_out,
                        children_results: r.children_results,
                    }
                } else {
                    r
                }
            }
        };
        chk_scope.emit_result(&r);
        r
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
        }
    }

    pub fn title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    pub fn param(mut self, name: impl Into<String>, value: ChkParamInstance) -> Self {
        self.actual_params.insert(name.into(), value);
        self
    }

    pub fn param_string(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.actual_params.insert(name.into(), ChkParamInstance::PkString(value.into()));
        self
    }

    pub fn param_int(mut self, name: impl Into<String>, value: i64) -> Self {
        self.actual_params.insert(name.into(), ChkParamInstance::PkInt(value));
        self
    }

    pub fn param_bool(mut self, name: impl Into<String>, value: bool) -> Self {
        self.actual_params.insert(name.into(), ChkParamInstance::PkBool(value));
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

    pub fn build(self) -> ChkInstance<'a> {
        ChkInstance {
            title: self.title,
            check_def: self.check_def,
            actual_params: self.actual_params,
            children: self.children,
            negated: self.negated,
            is_retrying: self.is_retrying,
            instance_id: self.instance_id,
        }
    }
}