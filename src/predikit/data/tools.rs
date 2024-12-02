// Copyright (c) 2025 Dave Parfitt

// add debug

use crate::predikit::data::ChkFormalParam;

use super::params::ChkActualParams;

#[derive(Debug, Clone)]
pub struct ToolDef {
    pub tool_name: String,
    pub template_params: ChkActualParams, // these turn into actual params at runtime
    pub instance_params: Vec<ChkFormalParam>,
    pub shell: Option<String>, // internally will default to "sh -c", maybe you want to us ksh?
    pub accepts_retry: bool,
}
