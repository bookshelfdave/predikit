// Copyright (c) 2025 Dave Parfitt

pub mod events;
pub mod instance;
pub mod params;
pub mod tools;

use crate::predikit::functions::builtin;
use instance::{ChkInstance, ChkResult, RunEnv};
use params::{ChkActualParam, ChkActualParams, ChkParamInternalValue};
use std::collections::HashMap;
use std::fmt;

type ChkFunctionParams = HashMap<String, ChkActualParam>;
type ChkFn = fn(&RunEnv, &ChkFunctionParams, &ChkInstance) -> ChkResult;

// for convenience!
pub type ChkFormalParams = HashMap<String, ChkFormalParam>;

#[derive(Debug, Clone)]
pub struct ChkDef {
    pub name: String,
    pub check_fn: ChkFn,
    // note: these are just the initial formal params.
    // See:  ChkInstance.materialized_formal_params,
    // which has additional formal parameters added to it
    // after the check is parsed
    pub formal_params: ChkFormalParams,
    pub accepts_children: bool,
    pub is_group: bool,
    pub template_params: Option<ChkActualParams>,
    pub is_query: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChkParamType {
    PkString,
    PkInt,
    PkBool,
    PkTypeName,
    PkDuration,
    PkPath,
}

impl ChkParamType {
    const STRING_TYPE: &'static str = "String";
    const INT_TYPE: &'static str = "Int";
    const BOOL_TYPE: &'static str = "Bool";
    const TYPE_NAME_TYPE: &'static str = "Typename";
    const DURATION_TYPE: &'static str = "Duration";
    const PATH_TYPE: &'static str = "Path";

    pub fn type_name(&self) -> &'static str {
        match self {
            ChkParamType::PkString => Self::STRING_TYPE,
            ChkParamType::PkInt => Self::INT_TYPE,
            ChkParamType::PkBool => Self::BOOL_TYPE,
            ChkParamType::PkTypeName => Self::TYPE_NAME_TYPE,
            ChkParamType::PkDuration => Self::DURATION_TYPE,
            ChkParamType::PkPath => Self::PATH_TYPE,
        }
    }
}

impl fmt::Display for ChkParamType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct ChkFormalParam {
    pub name: String,
    pub required: bool,
    pub param_type: ChkParamType,
    pub param_default: Option<ChkParamInternalValue>,
}

impl Default for ChkFormalParam {
    fn default() -> Self {
        Self {
            name: "".to_owned(),
            required: false,
            param_type: ChkParamType::PkString,
            param_default: None,
        }
    }
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
            if self.required {
                "required"
            } else {
                "optional"
            },
            self.param_type,
            def,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedDuration {
    pub duration: std::time::Duration,
    pub duration_str: String,
}

impl fmt::Display for ParsedDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.duration_str)
    }
}

impl ParsedDuration {
    pub fn new(duration: std::time::Duration, duration_str: String) -> Self {
        Self {
            duration,
            duration_str,
        }
    }
}

// mostly for ToolDef
pub fn formal_params_to_map(params: &Vec<ChkFormalParam>) -> ChkFormalParams {
    let mut m: HashMap<String, ChkFormalParam> = HashMap::new();
    for fp in params {
        m.insert(fp.name.clone(), fp.clone());
    }
    m
}

// The compiler is sad when these are not pub
pub struct FParamsBuilderStateInParam;
pub struct FParamsBuilderStateInBuilder;

// Define the builder for collecting ChkFormalParam into a HashMap
// This code is a bit janky, I'm not sure how I feel about it.
// I read about using marker traits in "Rust for Rustaceans", page 34,
// but haven't seen a real world example like this, so this is my first try
// at a nested builder pattern. Maybe it's a bad idea?
pub struct FParamsBuilder<T> {
    map: ChkFormalParams,
    current_param: Option<FParamBuilder>,
    state: T,
}

impl Default for FParamsBuilder<FParamsBuilderStateInBuilder> {
    fn default() -> Self {
        Self::new()
    }
}

impl FParamsBuilder<FParamsBuilderStateInBuilder> {
    // Create a new instance of the builder
    pub fn empty() -> ChkFormalParams {
        HashMap::new()
    }
    pub fn new() -> FParamsBuilder<FParamsBuilderStateInBuilder> {
        FParamsBuilder {
            map: HashMap::new(),
            current_param: None,
            state: FParamsBuilderStateInBuilder,
        }
    }

    // just to get the compiler to be quiet
    pub fn get_state(&self) -> &FParamsBuilderStateInBuilder {
        &self.state
    }
    pub fn add_param(
        self,
        name: impl Into<String>,
        param_type: ChkParamType,
    ) -> FParamsBuilder<FParamsBuilderStateInParam> {
        FParamsBuilder {
            map: self.map,
            current_param: Some(FParamBuilder::new(name).param_type(param_type)),
            state: FParamsBuilderStateInParam,
        }
    }

    // Add a ChkFormalParam to the HashMap
    // fn add_param(mut self, param: ChkFormalParam) -> Self {
    //     self.map.insert(param.name.clone(), param);
    //     self
    // }

    // Build the HashMap
    pub fn build(mut self) -> ChkFormalParams {
        self.map.shrink_to_fit();
        self.map
    }
}

impl FParamsBuilder<FParamsBuilderStateInParam> {
    pub fn finish_param(mut self) -> FParamsBuilder<FParamsBuilderStateInBuilder> {
        // calling build() adds the param that was just created to the
        // internal hashmap and changes state back to "builder" mode.
        let name = self.current_param.as_ref().unwrap().name.clone();
        let param = self.current_param.take().unwrap().build();
        self.map.insert(name, param);

        FParamsBuilder {
            map: self.map,
            current_param: None,
            state: FParamsBuilderStateInBuilder,
        }
    }

    pub fn required(mut self) -> Self {
        self.current_param.as_mut().unwrap().required = true;
        self
    }

    pub fn not_required(mut self) -> Self {
        self.current_param.as_mut().unwrap().required = false;
        self
    }

    pub fn pk_int(mut self) -> Self {
        self.current_param.as_mut().unwrap().param_type = ChkParamType::PkInt;
        self
    }

    pub fn pk_bool(mut self) -> Self {
        self.current_param.as_mut().unwrap().param_type = ChkParamType::PkBool;
        self
    }

    pub fn pk_string(mut self) -> Self {
        self.current_param.as_mut().unwrap().param_type = ChkParamType::PkString;
        self
    }

    pub fn default_value(mut self, default: ChkParamInternalValue) -> Self {
        self.current_param.as_mut().unwrap().param_default = Some(default);
        self
    }
}

pub struct FParamBuilder {
    name: String,
    required: bool,
    param_type: ChkParamType,
    param_default: Option<ChkParamInternalValue>,
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

    pub fn pk_duration(mut self) -> Self {
        self.param_type = ChkParamType::PkDuration;
        self
    }

    pub fn param_type(mut self, param_type: ChkParamType) -> Self {
        self.param_type = param_type;
        self
    }

    pub fn default_value(mut self, default: ChkParamInternalValue) -> Self {
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

#[derive(Debug, Default)]
pub struct ChkDefRegistry {
    pub check_fns: HashMap<String, ChkDef>,
    pub group_fns: HashMap<String, ChkDef>,
}

impl ChkDefRegistry {
    pub fn new_with_builtins() -> ChkDefRegistry {
        let mut reg = ChkDefRegistry::default();
        for check_def in builtin::define_builtins() {
            reg.register_fn(check_def.name.clone(), check_def);
        }
        for agg_def in builtin::define_aggs() {
            reg.register_group_fn(agg_def.name.clone(), agg_def);
        }

        reg
    }
    pub fn register_fn(&mut self, name: String, check_def: ChkDef) {
        self.check_fns.insert(name.clone(), check_def);
    }

    pub fn get_fn(&self, name: &str) -> Option<&ChkDef> {
        self.check_fns.get(name)
    }

    pub fn register_group_fn(&mut self, name: String, check_def: ChkDef) {
        self.group_fns.insert(name.clone(), check_def);
    }

    pub fn get_group_fn(&self, name: &str) -> Option<&ChkDef> {
        self.group_fns.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_builder() {
        let params = FParamsBuilder::new()
            .add_param("hello", ChkParamType::PkString)
            .required()
            .finish_param()
            .add_param("world", ChkParamType::PkInt)
            .not_required()
            .finish_param()
            .build();

        println!("{:#?}", params);
        assert!(params.contains_key("hello"));
        assert!(params.contains_key("world"));

        if let Some(param) = params.get("hello") {
            assert_eq!(param.param_type, ChkParamType::PkString);
            assert!(param.required);
        }

        if let Some(param) = params.get("world") {
            assert_eq!(param.param_type, ChkParamType::PkInt);
            assert!(!param.required);
        }
    }
}
