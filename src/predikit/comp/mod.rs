pub mod pkparser;

use crate::predikit::comp::pkparser::{PkCheckDef, PkCheckParams, PkChild, PkFile, PkGroup, PkTypedParamValue};
use crate::predikit::data::instance::ChkInstance;
use crate::predikit::data::{ChkActualParams, ChkDefRegistry, ChkParamInstance};
use log::{debug, error};
use peginator::PegParser;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// decouple the peginator AST from the rest of the code
pub struct FuncReg {
    pub reg: ChkDefRegistry,
}

impl FuncReg {
    pub fn new() -> Self {
        FuncReg {
            reg: ChkDefRegistry::new_with_builtins(),
        }
    }
}

pub struct IRMapCtx {
    pub next_id: usize,
}


impl IRMapCtx {
    pub fn new() -> Self {
        IRMapCtx {
            next_id: 1000
        }
    }

    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub fn file_to_ir<'a>(funs: &'a FuncReg, ctx: &mut IRMapCtx, cf: PkFile) -> Result<Vec<ChkInstance<'a>>, Box<dyn std::error::Error>> {
    let mut chks: Vec<ChkInstance> = Vec::new();
    for group in cf.groups {
        chks.push(group_to_ir(funs, ctx, &group));
    }
    Ok(chks)
}

fn check_def_to_ir<'a>(funs: &'a FuncReg, ctx: &mut IRMapCtx, cd: &PkCheckDef) -> ChkInstance<'a> {
    //let title = get_group_title(&group);
    let title = None;
    let negated = cd.negate.is_some();
    let is_retrying = false;
    if is_retrying {
        panic!("Check retries not supported");
    }

    let actual_params = params_to_ir(funs, &cd.params);
    let check_fn = funs.reg.get_check(&cd.check_fn);
    if check_fn.is_none() {
        panic!("Check function {} not found", cd.check_fn);
    }
    ChkInstance {
        title,
        check_def: check_fn.unwrap(),
        actual_params,
        children: vec![],
        negated,
        is_retrying: false, // not supported yet
        instance_id: ctx.next_id(),
    }
}

fn params_to_ir(_: &FuncReg, params_root: &PkCheckParams) -> ChkActualParams {
    let mut actual_params: ChkActualParams = ChkActualParams::new();
    for param in &params_root.params {
        let param_name = param.param_name[1..].to_string();
        let param_val = match &param.param_typed_value {
            PkTypedParamValue::PkParamTypeString(s) => {
                let tmp = s.clone();
                // strip leading and trailing quotes from tmp
                let s = tmp.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
                ChkParamInstance::PkString(s.to_owned())
            }
            PkTypedParamValue::PkParamTypeInt(i) => ChkParamInstance::PkInt(i.parse::<i64>().unwrap()),
            PkTypedParamValue::PkParamTypeBool(b) => ChkParamInstance::PkBool(b.parse::<bool>().unwrap()),
        };
        actual_params.insert(param_name, param_val);
    }
    actual_params
}

fn group_to_ir<'a>(funs: &'a FuncReg, ctx: &mut IRMapCtx, group: &PkGroup) -> ChkInstance<'a> {
    let title = get_group_title(&group);
    let agg_type = group.agg_type.to_string();
    let negated = false; // groups can't be negated
    let is_retrying = false;
    if negated {
        panic!("Groups can't be negated");
    }
    if is_retrying {
        panic!("Group retries not supported");
    }
    let children: Vec<ChkInstance> = group.children.iter().map(|c| {
        match c {
            PkChild::PkCheckDef(cd) => check_def_to_ir(funs, ctx, cd),
            PkChild::PkGroup(g) => group_to_ir(funs, ctx, g)
        }
    }).collect::<Vec<ChkInstance>>();

    let actual_params = params_to_ir(funs, &group.group_params);
    debug!("AGG TYPE: {}", agg_type);
    let agg_fn = funs.reg.get_agg(&agg_type);
    if agg_fn.is_none() {
        panic!("Unknown agg type: [{}]", agg_type);
    }

    ChkInstance {
        title,
        check_def: agg_fn.unwrap(),
        actual_params,
        children,
        negated,
        is_retrying: false, // not supported yet
        instance_id: ctx.next_id(),
    }
}

fn get_group_title(group: &PkGroup) -> Option<String> {
    let p: Vec<Option<String>> = group.group_params.params.iter().
        filter(|p| p.param_name == ":title").
        map(|p| {
            if let PkTypedParamValue::PkParamTypeString(s) = &p.param_typed_value {
                // strip the quotes from the string
                Some(s[1..(s.len() - 1)].to_string())
            } else {
                panic!("Expected a string");
            }
        }).take(1).collect();
    if p.len() > 0 {
        p[0].clone()
    } else {
        None
    }
}

pub fn compile_file(path: &Path) -> Result<PkFile, Box<dyn std::error::Error>> {
    // Open the file
    let mut file = File::open(path)?;

    // Create a String to hold the file contents
    let mut contents = String::new();

    // Read the file contents into the string
    file.read_to_string(&mut contents)?;

    let parsed_file = PkFile::parse(&contents);
    if let Err(err) = parsed_file {
        // for now...
        error!("Error parsing file: {} {}", path.display(), err);
        return Err(Box::new(err));
    }
    let parsed_file = parsed_file.unwrap();
    debug!("Parsed a file!");
    debug!("{:#?}", parsed_file);
    Ok(parsed_file)
}

