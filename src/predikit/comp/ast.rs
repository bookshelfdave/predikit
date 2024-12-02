// Copyright (c) 2025 Dave Parfitt

use std::collections::HashMap;

use crate::predikit::data::{instance::ContentAddress, params::ChkActualParam};

// These structs represent "raw" (pre-typechecked) Checks, Tools, and Parameters.
// Check (function) names may not be defined and types may be incorrect (among other things
// that could be incomplete).
// Structs can carry a ContentAddress. These should live and die in the Ast* structs or
// consumed via error message reporting during type checking.

#[derive(Debug, Clone)]
pub struct AstFile {
    pub filename: Option<String>,
    pub tools: AstFileTools,
    pub checks: AstFileChecks,
}

#[derive(Debug, Clone)]
pub struct AstFileChecks {
    pub filename: Option<String>,
    pub check_defs: Vec<AstCheckDef>,
}
impl AstFileChecks {
    pub fn new(filename: Option<String>, check_defs: Vec<AstCheckDef>) -> Self {
        Self {
            filename,
            check_defs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AstFileTools {
    pub filename: Option<String>,
    pub tool_defs: Vec<AstToolDef>,
}

impl AstFileTools {
    pub fn new(filename: Option<String>, tool_defs: Vec<AstToolDef>) -> Self {
        Self {
            tool_defs,
            filename,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TopLevelItem {
    Check(AstCheckDef),
    Group(AstCheckDef),
    Tool(AstToolDef),
}

#[derive(Debug, Clone)]
pub struct AstCheckDef {
    pub fn_name: String,
    pub is_negated: bool,
    pub is_retrying: bool,
    pub actual_params: AstActualParams,
    pub children: Vec<AstCheckDef>,
    pub content_address: ContentAddress,
    pub is_group: bool,
}

pub type AstActualParams = HashMap<String, ChkActualParam>;

pub type AstToolInstanceParams = HashMap<String, AstActualParams>;

#[derive(Debug, Clone)]
pub struct AstToolDefParams {
    // def_params contains params like "cmd_template" in `tool foo? { cmd_template: "foobar123"` ...
    pub template_params: AstActualParams,
    // instance params are the user defined parameters for a template that are passed in via a check instance
    // ex: cmd_to_run in `test my_tool? { cmd_to_run: "df -h"` ...
    pub instance_params: AstToolInstanceParams,
}

impl AstToolDefParams {
    pub fn new(def_params: AstActualParams, instance_params: AstToolInstanceParams) -> Self {
        Self {
            template_params: def_params,
            instance_params,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AstToolDef {
    pub name: String,
    pub content_address: ContentAddress,
    pub params: AstToolDefParams,
}
impl AstToolDef {
    pub fn new(name: String, content_address: ContentAddress, params: AstToolDefParams) -> Self {
        Self {
            name,
            content_address,
            params,
        }
    }
}
