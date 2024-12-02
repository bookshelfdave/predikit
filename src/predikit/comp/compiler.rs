// Copyright (c) 2025 Dave Parfitt

use std::ops::Range;

use log::debug;

use crate::predikit::{
    comp::CompilerErrorType,
    data::{instance::ChkInstance, tools::ToolDef, ChkDefRegistry, ChkFormalParam, ChkParamType},
    functions::builtin_tools::metadef_tool,
};

use super::{
    ast::{AstCheckDef, AstFileChecks, AstFileTools},
    CompileError, CompiledCheckFileOut,
};

// Note that this does NOT return a fully typechecked ChkInstance (hence private visibility).:
// Typechecking happens later. TODO: create a pre-typechecked ChkInstance type? Might not be worth it.
fn make_check_instance<'a>(
    cfo: &mut CompiledCheckFileOut,
    fns: &'a ChkDefRegistry,
    ast_check_def: AstCheckDef,
) -> Option<ChkInstance<'a>> {
    let mut children = Vec::new();
    for child in ast_check_def.children {
        let child_inst = make_check_instance(cfo, fns, child)?;
        children.push(child_inst);
    }

    let actual_params = ast_check_def.actual_params;

    let fn_def = if ast_check_def.is_group {
        fns.group_fns.get(&ast_check_def.fn_name)
    } else {
        fns.check_fns.get(&ast_check_def.fn_name)
    };

    if fn_def.is_none() {
        cfo.add_error(
            cfo.filename.clone(),
            ast_check_def.content_address,
            format!("Unknown function {}", ast_check_def.fn_name),
        );
        return None;
    }
    let fn_def = fn_def.unwrap();

    let mut inst = ChkInstance {
        title: None,
        fn_def,
        actual_params,
        materialized_formal_params: None,
        children,
        negated: ast_check_def.is_negated,
        is_retrying: ast_check_def.is_retrying,
        instance_id: cfo.next_id(),
        content_address: ast_check_def.content_address,
        is_query: false,
    };
    inst.materialize_formal_params();
    Some(inst)
}

fn typecheck_check_params(cfo: &mut CompiledCheckFileOut, inst: &ChkInstance) {
    debug!("Typechecking {}", inst.fn_def.name);

    // check for required params
    let mut not_found: Vec<String> = inst
        .materialized_formal_params
        .as_ref()
        .unwrap()
        .values()
        .filter(|fp| fp.required)
        .filter(|fp| !inst.actual_params.contains_key(&fp.name))
        .map(|fp| fp.name.clone())
        .collect();

    not_found.sort(); // make not found error messages appear in the same
                      // order every time they're raised
    if !not_found.is_empty() {
        for missing_param in not_found.iter() {
            let msg = format!(
                "Required parameter is missing for check {}: {}",
                inst.fn_def.name, missing_param
            );
            cfo.add_error(
                cfo.filename.clone(),
                inst.content_address.clone(),
                msg.clone(),
            );
        }
    }

    // look for unexpected / unknown parameters
    let unexpected_params: Vec<_> = inst
        .actual_params
        .iter()
        .filter(|ap| {
            inst.materialized_formal_params
                .as_ref()
                .unwrap()
                .get(ap.0)
                .is_none()
        })
        .collect();
    if !unexpected_params.is_empty() {
        for (k, _v) in unexpected_params {
            let msg = format!("Unexpected param \"{}\" for check: {}", k, inst.fn_def.name);
            cfo.add_error(cfo.filename.clone(), inst.content_address.clone(), msg);
        }
    }

    // check parameter types
    for (formal_param_name, formal_param) in inst.materialized_formal_params.as_ref().unwrap() {
        if let Some(actual_param) = inst.actual_params.get(formal_param_name) {
            // Path formal param can accept Path or String
            // String formal param can accept only String
            if formal_param.param_type == ChkParamType::PkPath {
                if !actual_param.is_coercible_to_path() {
                    let msg = format!(
                        "Invalid parameter type for check {} param '{}'. Got type {}, but expected type String or Path",
                        inst.fn_def.name,
                        formal_param_name,
                        actual_param.type_name(),
                    );
                    cfo.add_error(
                        cfo.filename.clone(),
                        actual_param.content_address.clone(),
                        msg,
                    );
                }
            } else if !actual_param.is_type(&formal_param.param_type) {
                let msg = format!(
                    "Invalid parameter type for check {} param '{}'. Got type {}, but expected type {}",
                    inst.fn_def.name,
                    formal_param_name,
                    actual_param.type_name(),
                    formal_param.param_type.type_name()
                );
                cfo.add_error(
                    cfo.filename.clone(),
                    actual_param.content_address.clone(),
                    msg,
                );
            }
        } else {
            debug!(
                "Missing non-required parameter {} for check {}",
                formal_param_name, inst.fn_def.name
            );
        }
    }
    for child in &inst.children {
        typecheck_check_params(cfo, child);
    }
}

pub fn compile_checks_to_asts<'chkdef>(
    fns: &'chkdef ChkDefRegistry,
    all_ast_file_checks: Vec<AstFileChecks>,
) -> Vec<CompiledCheckFileOut<'chkdef>> {
    let mut all_compiled_files = Vec::with_capacity(all_ast_file_checks.len());

    for ast_file_checks in all_ast_file_checks {
        let mut cfo = CompiledCheckFileOut::new(ast_file_checks.filename.clone());
        let maybe_insts: Vec<_> = ast_file_checks
            .check_defs
            .into_iter()
            .map(|ast_check_def| make_check_instance(&mut cfo, fns, ast_check_def))
            .collect();

        if maybe_insts.iter().any(|inst| inst.is_none()) {
            // bail out if a child instance can't compile
            // the cctx will have any compile errors from the child
            cfo.add_error(
                cfo.filename.clone(),
                Range { start: 0, end: 0 },
                "Child instance failed to compile".to_owned(),
            );
            // TODO: I'm not excited about the flow here
            all_compiled_files.push(cfo);
            continue;
        }

        let insts: Vec<ChkInstance<'chkdef>> = maybe_insts
            .into_iter()
            .map(|maybe_inst| {
                let mut i = maybe_inst.unwrap();
                i.materialize_formal_params();
                i
            })
            .collect();
        for inst in insts {
            typecheck_check_params(&mut cfo, &inst);
            cfo.add_instance(inst);
        }
        all_compiled_files.push(cfo);
    }
    all_compiled_files
}

pub fn make_tools(
    fns: &mut ChkDefRegistry,
    ast_file_tools: Vec<AstFileTools>,
) -> Vec<CompileError> {
    let mut compile_errors = vec![];

    for ast_file_tool in ast_file_tools {
        for tool_def in ast_file_tool.tool_defs {
            let mut instance_params: Vec<ChkFormalParam> = vec![];
            let template_params = tool_def.params.template_params;
            // process each $foo parameter. It needs to have a "type" field and a "required" field
            for (k, map) in tool_def.params.instance_params {
                let unknown_keys: Vec<_> = map
                    .keys()
                    .filter(|k| *k != "type" && *k != "required")
                    .collect();

                if !unknown_keys.is_empty() {
                    compile_errors.push(CompileError {
                        filename: ast_file_tool.filename.clone(),
                        message: format!(
                            "Unknown keys found in tool instance params: {:?}",
                            unknown_keys
                        ),
                        content_address: tool_def.content_address.clone(),
                        error_type: CompilerErrorType::Error,
                    });
                }

                let mut valid_prop = true;
                if let Some(p) = map.get("type") {
                    if !p.is_type(&ChkParamType::PkTypeName) {
                        compile_errors.push(CompileError {
                            filename: ast_file_tool.filename.clone(),
                            message: format!("Invalid type {} for param 'type', should be a type name (String, Int, Bool)", p.type_name()),
                            content_address: p.content_address.clone(),
                            error_type: CompilerErrorType::Error,
                        });
                        valid_prop = false;
                    }
                } else {
                    compile_errors.push(CompileError {
                        filename: ast_file_tool.filename.clone(),
                        message: "Missing tool parameter 'type'".to_string(),
                        content_address: tool_def.content_address.clone(),
                        error_type: CompilerErrorType::Error,
                    });
                    valid_prop = false;
                }

                if let Some(p) = map.get("required") {
                    if !p.is_type(&ChkParamType::PkBool) {
                        compile_errors.push(CompileError {
                            filename: ast_file_tool.filename.clone(),
                            message: format!(
                                "Invalid type {} for param 'required', should be a bool",
                                p.type_name()
                            ),
                            content_address: p.content_address.clone(),
                            error_type: CompilerErrorType::Error,
                        });
                        valid_prop = false;
                    }
                } else {
                    compile_errors.push(CompileError {
                        filename: ast_file_tool.filename.clone(),
                        message: "Missing tool parameter 'required'".to_string(),
                        content_address: tool_def.content_address.clone(),
                        error_type: CompilerErrorType::Error,
                    });
                    valid_prop = false;
                }

                if valid_prop {
                    let type_name = map.get("type").unwrap();
                    let required = map.get("required").unwrap();
                    instance_params.push(ChkFormalParam {
                        name: k,
                        required: required.get_bool(),
                        param_type: ChkParamType::from(type_name.get_named_type()),
                        param_default: None,
                    });
                }
            }
            if !compile_errors.is_empty() {
                // gather up error messages, but don't register any tools
                continue;
            }
            let td = ToolDef {
                tool_name: tool_def.name,
                template_params,
                instance_params,
                shell: None,         // TODO
                accepts_retry: true, // TODO
            };
            fns.register_fn(td.tool_name.clone(), metadef_tool(&td));
        }
    }
    compile_errors
}

#[cfg(test)]
mod tests {
    use crate::predikit::{
        comp::ast::AstActualParams,
        data::{
            instance::ChkResult,
            params::{ChkActualParam, ChkActualParams},
            ChkDef, FParamsBuilder,
        },
    };

    use super::*;

    fn get_test_only_check_fn() -> ChkDef {
        let chk_def = ChkDef {
            name: "for_testing_only!".to_owned(),
            is_group: false,
            accepts_children: false,
            template_params: None,
            is_query: false,
            formal_params: FParamsBuilder::new()
                .add_param("str_param", ChkParamType::PkString)
                .required()
                .finish_param()
                .add_param("int_param", ChkParamType::PkInt)
                .required()
                .finish_param()
                .add_param("bool_param", ChkParamType::PkBool)
                .required()
                .finish_param()
                .add_param("not_required_string", ChkParamType::PkString)
                .not_required()
                .finish_param()
                .build(),
            check_fn: |_, params: &ChkActualParams, _| -> ChkResult {
                let str_param = params.get("str_param").unwrap();
                let int_param = params.get("int_param").unwrap();
                let bool_param = params.get("bool_param").unwrap();
                let non_req_str_param = params.get("non_required_string");

                println!("str_param {:#?}", str_param);
                println!("int_param {:#?}", int_param);
                println!("bool_param {:#?}", bool_param);
                println!("non_required_string {:#?}", non_req_str_param);

                ChkResult {
                    result: Ok(true),
                    process_out: None,
                    children_results: None,
                }
            },
        };

        chk_def
    }

    fn get_test_only_check_fn_with_paths() -> ChkDef {
        let chk_def = ChkDef {
            name: "for_testing_paths_only!".to_owned(),
            is_group: false,
            accepts_children: false,
            template_params: None,

            is_query: false,
            formal_params: FParamsBuilder::new()
                .add_param("str_param1", ChkParamType::PkString)
                .required()
                .finish_param()
                .add_param("str_param2", ChkParamType::PkString)
                .required()
                .finish_param()
                .add_param("path_param1", ChkParamType::PkPath)
                .required()
                .finish_param()
                .add_param("path_param2", ChkParamType::PkPath)
                .required()
                .finish_param()
                .build(),
            check_fn: |_, params: &ChkActualParams, _| -> ChkResult {
                let str_param1 = params.get("str_param1").unwrap();
                let str_param2 = params.get("str_param2").unwrap();
                let path_param1 = params.get("path_param1").unwrap();
                let path_param2 = params.get("path_param2").unwrap();

                println!("str_param1 {:#?}", str_param1);
                println!("str_param2 {:#?}", str_param2);
                println!("path_param1 {:#?}", path_param1);
                println!("path_param2 {:#?}", path_param2);

                ChkResult {
                    result: Ok(true),
                    process_out: None,
                    children_results: None,
                }
            },
        };

        chk_def
    }

    #[test]
    fn test_make_check_instance_simple() {
        let mut ccfo = CompiledCheckFileOut::new(Some("./foo.pk".to_string()));
        let fns = ChkDefRegistry::new_with_builtins();

        let mut actual_params = AstActualParams::new();
        let path = ChkActualParam::new_string("path".to_string(), file!().to_string(), 0..0);
        actual_params.insert("path".to_string(), path);

        let ast_check_def = AstCheckDef {
            fn_name: "exists?".to_string(),
            is_negated: false,
            is_retrying: false,
            actual_params,
            children: vec![],
            content_address: 0..0,
            is_group: false,
        };

        let result = make_check_instance(&mut ccfo, &fns, ast_check_def);
        assert!(result.is_some());
        assert_eq!(ccfo.errors.len(), 0);
    }

    #[test]
    fn test_make_check_instance_invalid_function() {
        let mut ccfo = CompiledCheckFileOut::new(Some("foo.pk".to_string()));
        let fns = ChkDefRegistry::new_with_builtins();

        let mut actual_params = AstActualParams::new();
        let path = ChkActualParam::new_string("path".to_string(), file!().to_string(), 0..0);
        actual_params.insert("path".to_string(), path);

        let ast_check_def = AstCheckDef {
            fn_name: "foo?".to_string(),
            is_negated: false,
            is_retrying: false,
            actual_params,
            children: vec![],
            content_address: 0..0,
            is_group: false,
        };

        let result = make_check_instance(&mut ccfo, &fns, ast_check_def);
        assert!(result.is_none());
        assert_eq!(ccfo.errors.len(), 1);

        let compiler_err = ccfo.errors.first().unwrap();
        assert_eq!(compiler_err.filename, Some("foo.pk".to_string()));
        assert_eq!(compiler_err.error_type, CompilerErrorType::Error);
        assert_eq!(compiler_err.content_address, 0..0);

        assert_eq!(
            ccfo.errors.first().unwrap().message,
            "Unknown function foo?".to_string()
        )
    }

    #[test]
    fn test_make_check_instance_with_children() {
        let mut ccfo = CompiledCheckFileOut::new(Some("foo.pk".to_string()));
        let fns = ChkDefRegistry::new_with_builtins();

        let mut chk_actual_params = AstActualParams::new();
        let path = ChkActualParam::new_string("path".to_string(), file!().to_string(), 0..0);
        chk_actual_params.insert("path".to_string(), path);

        let ast_check_def = AstCheckDef {
            fn_name: "exists?".to_string(),
            is_negated: false,
            is_retrying: false,
            actual_params: chk_actual_params,
            children: vec![],
            content_address: 0..0,
            is_group: false,
        };

        let mut group_actual_params = AstActualParams::new();
        let path = ChkActualParam::new_string("title".to_string(), "Test title".to_string(), 0..0);
        group_actual_params.insert("title".to_string(), path);

        let all_check_def = AstCheckDef {
            fn_name: "all".to_string(),
            is_negated: false,
            is_retrying: false,
            actual_params: group_actual_params,
            children: vec![ast_check_def],
            content_address: 0..0,
            is_group: true,
        };

        let result = make_check_instance(&mut ccfo, &fns, all_check_def);
        assert!(result.is_some());
        assert_eq!(ccfo.errors.len(), 0);
        let inst = result.unwrap();
        assert_eq!("all".to_string(), inst.fn_def.name);
        let child = inst.children.first().unwrap();
        assert_eq!("exists?".to_string(), child.fn_def.name);
    }

    #[test]
    fn test_typecheck_check_params() {
        // simple case, should typecheck successfully
        let mut ccfo = CompiledCheckFileOut::new(Some("foo.pk".to_string()));
        let mut fns = ChkDefRegistry::new_with_builtins();
        fns.register_fn("for_testing_only!".to_string(), get_test_only_check_fn());

        let mut actual_params = AstActualParams::new();
        actual_params.insert(
            "str_param".to_string(),
            ChkActualParam::new_string("str_param".to_string(), file!().to_string(), 0..5),
        );
        actual_params.insert(
            "int_param".to_string(),
            ChkActualParam::new_int("int_param".to_string(), 99, 25..30),
        );
        actual_params.insert(
            "bool_param".to_string(),
            ChkActualParam::new_bool("bool_param".to_string(), false, 50..55),
        );

        let ast_check_def = AstCheckDef {
            fn_name: "for_testing_only!".to_string(),
            is_negated: false,
            is_retrying: false,
            actual_params,
            children: vec![],
            content_address: 0..0,
            is_group: false,
        };

        let inst = make_check_instance(&mut ccfo, &fns, ast_check_def).unwrap();
        typecheck_check_params(&mut ccfo, &inst);
        assert!(ccfo.errors.is_empty());
    }

    #[test]
    fn test_typecheck_check_missing_required_params() {
        let mut ccfo = CompiledCheckFileOut::new(Some("foo.pk".to_string()));
        let mut fns = ChkDefRegistry::new_with_builtins();
        fns.register_fn("for_testing_only!".to_string(), get_test_only_check_fn());

        let mut actual_params = AstActualParams::new();
        actual_params.insert(
            "str_param".to_string(),
            ChkActualParam::new_string("str_param".to_string(), file!().to_string(), 0..5),
        );

        let ast_check_def = AstCheckDef {
            fn_name: "for_testing_only!".to_string(),
            is_negated: false,
            is_retrying: false,
            actual_params,
            children: vec![],
            content_address: 0..0,
            is_group: false,
        };

        let inst = make_check_instance(&mut ccfo, &fns, ast_check_def).unwrap();
        typecheck_check_params(&mut ccfo, &inst);
        assert_eq!(2, ccfo.errors.len());

        let bool_param_err = ccfo.errors.first().unwrap();
        assert_eq!(
            bool_param_err.message,
            "Required parameter is missing for check for_testing_only!: bool_param".to_string()
        );

        let int_param_err = ccfo.errors.get(1).unwrap();
        assert_eq!(
            int_param_err.message,
            "Required parameter is missing for check for_testing_only!: int_param".to_string()
        );
    }

    #[test]
    fn test_typecheck_check_extra_params() {
        let mut ccfo = CompiledCheckFileOut::new(Some("foo.pk".to_string()));
        let mut fns = ChkDefRegistry::new_with_builtins();
        fns.register_fn("for_testing_only!".to_string(), get_test_only_check_fn());

        let mut actual_params = AstActualParams::new();
        actual_params.insert(
            "str_param".to_string(),
            ChkActualParam::new_string("str_param".to_string(), file!().to_string(), 0..5),
        );
        actual_params.insert(
            "int_param".to_string(),
            ChkActualParam::new_int("int_param".to_string(), 99, 25..30),
        );
        actual_params.insert(
            "bool_param".to_string(),
            ChkActualParam::new_bool("bool_param".to_string(), false, 50..55),
        );

        actual_params.insert(
            "extra_param".to_string(),
            ChkActualParam::new_bool("extra_param".to_string(), false, 75..80),
        );

        let ast_check_def = AstCheckDef {
            fn_name: "for_testing_only!".to_string(),
            is_negated: false,
            is_retrying: false,
            actual_params,
            children: vec![],
            content_address: 0..0,
            is_group: false,
        };

        let inst = make_check_instance(&mut ccfo, &fns, ast_check_def).unwrap();
        typecheck_check_params(&mut ccfo, &inst);
        println!("{:#?}", ccfo);
        assert_eq!(1, ccfo.errors.len());
        let extra_param_err = ccfo.errors.first().unwrap();
        assert_eq!(
            "Unexpected param \"extra_param\" for check: for_testing_only!",
            extra_param_err.message
        );
    }

    #[test]
    fn test_typecheck_check_invalid_param_type() {
        let mut ccfo = CompiledCheckFileOut::new(Some("foo.pk".to_string()));
        let mut fns = ChkDefRegistry::new_with_builtins();
        fns.register_fn("for_testing_only!".to_string(), get_test_only_check_fn());

        let mut actual_params = AstActualParams::new();
        actual_params.insert(
            "int_param".to_string(), // note that int_param us using new_string
            ChkActualParam::new_string("int_param".to_string(), file!().to_string(), 0..5),
        );
        actual_params.insert(
            "str_param".to_string(), // note that str_param is using new_int
            ChkActualParam::new_int("str_param".to_string(), 99, 25..30),
        );
        actual_params.insert(
            "bool_param".to_string(),
            ChkActualParam::new_bool("bool_param".to_string(), false, 50..55),
        );

        let ast_check_def = AstCheckDef {
            fn_name: "for_testing_only!".to_string(),
            is_negated: false,
            is_retrying: false,
            actual_params,
            children: vec![],
            content_address: 0..0,
            is_group: false,
        };

        let inst = make_check_instance(&mut ccfo, &fns, ast_check_def).unwrap();
        typecheck_check_params(&mut ccfo, &inst);
        println!("{:#?}", ccfo);
        assert_eq!(2, ccfo.errors.len());

        let expected_int_err_expected =
            "Invalid parameter type for check for_testing_only! param 'int_param'. Got type String, but expected type Int".to_string();
        assert!(ccfo
            .errors
            .iter()
            .find(|e| e.message == expected_int_err_expected)
            .is_some());

        let expected_string_err_expected =
            "Invalid parameter type for check for_testing_only! param 'str_param'. Got type Int, but expected type String".to_string();
        assert!(ccfo
            .errors
            .iter()
            .find(|e| e.message == expected_string_err_expected)
            .is_some());
    }

    #[test]
    fn test_typecheck_paths() {
        let mut ccfo = CompiledCheckFileOut::new(Some("foo.pk".to_string()));
        let mut fns = ChkDefRegistry::new_with_builtins();
        fns.register_fn(
            "for_testing_paths_only!".to_string(),
            get_test_only_check_fn_with_paths(),
        );

        let mut actual_params = AstActualParams::new();
        actual_params.insert(
            "path_param1".to_string(),
            ChkActualParam::new_path("path_param1".to_string(), file!().to_string(), 0..5),
        );

        // Path param type CAN accept strings
        actual_params.insert(
            "path_param2".to_string(),
            ChkActualParam::new_string("path_param2".to_string(), file!().to_string(), 6..10),
        );

        // String param type CAN'T accept paths
        actual_params.insert(
            "str_param1".to_string(),
            ChkActualParam::new_path("str_param1".to_string(), file!().to_string(), 25..30),
        );

        actual_params.insert(
            "str_param2".to_string(),
            ChkActualParam::new_string("str_param2".to_string(), file!().to_string(), 31..35),
        );

        let ast_check_def = AstCheckDef {
            fn_name: "for_testing_paths_only!".to_string(),
            is_negated: false,
            is_retrying: false,
            actual_params,
            children: vec![],
            content_address: 0..0,
            is_group: false,
        };

        let inst = make_check_instance(&mut ccfo, &fns, ast_check_def).unwrap();
        typecheck_check_params(&mut ccfo, &inst);
        assert_eq!(1, ccfo.errors.len());

        assert_eq!(ccfo.errors.first().unwrap().message,
            "Invalid parameter type for check for_testing_paths_only! param 'str_param1'. Got type Path, but expected type String".to_string());
    }

    #[test]
    fn test_compile_checks_to_asts() {
        // TODO
    }

    #[test]
    fn test_make_tools() {
        // TODO
    }
}
