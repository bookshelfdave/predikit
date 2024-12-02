// Copyright (c) 2025 Dave Parfitt

use crate::predikit::data::instance::{ChkProcessOut, ChkResult};
use crate::predikit::data::params::ChkActualParams;
use crate::predikit::data::{ChkDef, ChkParamType, FParamsBuilder};
use log::debug;
use std::path::Path;

#[macro_export]
macro_rules! fparam_str {
    ($name:expr) => {
        ChkFormalParam {
            name: $name.to_owned(),
            required: true,
            param_type: ChkParamType::PkString,
            param_default: None,
        }
    };
    ($name:expr, required = $req:expr) => {
        ChkFormalParam {
            name: $name.to_owned(),
            required: $req,
            param_type: ChkParamType::PkString,
            param_default: None,
        }
    };

    ($name:expr, required = $req:expr, default = $default:expr) => {
        ChkFormalParam {
            name: $name.to_owned(),
            required: $req,
            param_type: ChkParamType::PkString,
            param_default: Some(PkString($default)),
        }
    };
}
#[macro_export]
macro_rules! fparam {
    ($name:expr) => {
        ChkFormalParam {
            name: $name.to_owned(),
            required: true,
            param_type: ChkParamType::PkString,
            param_default: None,
        }
    };
    ($name:expr, required = $req:expr) => {
        ChkFormalParam {
            name: $name.to_owned(),
            required: $req,
            param_type: ChkParamType::PkString,
            param_default: None,
        }
    };
    ($name:expr, type = $type:expr) => {
        ChkFormalParam {
            name: $name.to_owned(),
            required: true,
            param_type: $type,
            param_default: None,
        }
    };
    ($name:expr, required = $req:expr, type = $type:expr) => {
        ChkFormalParam {
            name: $name.to_owned(),
            required: $req,
            param_type: $type,
            param_default: None,
        }
    };
}

pub fn cd_file_exists() -> ChkDef {
    ChkDef {
        name: "exists?".to_owned(),
        is_group: false,
        accepts_children: false,
        template_params: None,
        is_query: false,
        formal_params: FParamsBuilder::new()
            .add_param("path", ChkParamType::PkPath)
            .required()
            .finish_param()
            .build(),
        check_fn: |_, params: &ChkActualParams, _| -> ChkResult {
            let p0 = params.get("path").unwrap();
            let path = p0.get_path();
            if path.is_err() {
                return ChkResult {
                    result: Err(path.unwrap_err()),
                    process_out: None,
                    children_results: None,
                };
            }
            let path = path.unwrap();
            let o = Path::new(&path).exists();
            debug!("exists path: {} == {}", path, o);
            ChkResult {
                result: Ok(o),
                process_out: None,
                children_results: None,
            }
        },
    }
}

pub fn cd_file_is_executable() -> ChkDef {
    use crate::predikit::data::instance::ChkResult;
    use is_executable::IsExecutable;
    ChkDef {
        name: "executable?".to_owned(),
        is_group: false,
        accepts_children: false,
        template_params: None,
        is_query: false,
        formal_params: FParamsBuilder::new()
            .add_param("path", ChkParamType::PkPath)
            .required()
            .finish_param()
            .build(),
        check_fn: |_, params: &ChkActualParams, _| -> ChkResult {
            let path = params.get("path").unwrap();
            let is_exec = Path::new(&path.get_string()).is_executable();
            ChkResult {
                result: Ok(is_exec),
                process_out: None,
                children_results: None,
            }
        },
    }
}

pub fn cd_file_is_on_path() -> ChkDef {
    use which::which;
    ChkDef {
        name: "on_path?".to_owned(),
        is_group: false,
        accepts_children: false,
        template_params: None,
        is_query: false,
        formal_params: FParamsBuilder::new()
            .add_param("path", ChkParamType::PkPath)
            .required()
            .finish_param()
            .build(),
        check_fn: |_, params: &ChkActualParams, _| -> ChkResult {
            let path = params.get("path").unwrap();

            let w = which(path.get_string());
            ChkResult {
                result: Ok(w.is_ok()),
                process_out: None,
                children_results: None,
            }
        },
    }
}

pub fn cd_shell() -> ChkDef {
    ChkDef {
        name: "shell".to_owned(),
        is_group: false,
        accepts_children: false,
        template_params: None,
        is_query: false,
        formal_params: FParamsBuilder::new()
            .add_param("cmd", ChkParamType::PkString)
            .required()
            .finish_param()
            .build(),
        check_fn: |_, params: &ChkActualParams, _| -> ChkResult {
            let cmd = params.get("cmd").unwrap();
            let cmd_result = std::process::Command::new("sh")
                .arg("-c")
                .arg(cmd.get_string())
                .output();

            match cmd_result {
                Ok(output) => {
                    debug!("SHELL RESULT = {}", &output.status.code().unwrap());
                    debug!("SHELL RESULT 2= {}", &output.status.success());
                    ChkResult {
                        result: Ok(output.status.success()),
                        process_out: Some(ChkProcessOut {
                            stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
                            stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                            exit_code: output.status.code(),
                        }),
                        children_results: None,
                    }
                }
                Err(e) => ChkResult {
                    result: Err(e.to_string()),
                    process_out: None,
                    children_results: None,
                },
            }
        },
    }
}

// pub fn cd_shell_exec() -> CheckDef {
//     CheckDef {
//         short_name: "exec".to_owned(),
//         formal_params: vec![CheckParam::required("cmdline".to_owned()),
//                             CheckParam::optional("shell".to_owned()),
//         ],
//         check_fn: |_, params: &HashMap<String, String>, _| -> CheckResult {
//             let cmd = params.get("cmdline").unwrap();
//
//             let cmd_output = if params.get("shell").is_some() {
//                 let shell = params.get("shell").unwrap();
//                 debug!("Using a custom shell for cmd.exec: [{}]", &shell);
//
//                 // TODO: this needs error checking
//                 let chunks = shell.split_whitespace().collect::<Vec<_>>();
//                 let mut p = std::process::Command::new(chunks[0]);
//                 for x in chunks.iter().skip(1) {
//                     debug!("Adding arg [{}]", &x);
//                     p.arg(x);
//                 }
//                 debug!("Using cmd [{}]", &cmd);
//                 p.arg(cmd);
//                 p.output()
//             } else {
//                 // TODO: this definitely needs a refactor, but lets see if it works first
//                 std::process::Command::new("sh")
//                     .arg("-c")
//                     .arg(cmd)
//                     .output()
//             };
//
//             match cmd_output {
//                 Ok(output) => {
//                     debug!("SHELL RESULT = {}", &output.status.code().unwrap());
//                     debug!("SHELL RESULT 2= {}", &output.status.success());
//                     CheckResult {
//                         result: Ok(output.status.success()),
//                         process_out: Some(ProcessOut {
//                             stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
//                             stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
//                             exit_code: output.status.code(),
//                         }),
//                         children_results: None,
//                     }
//                 }
//                 Err(e) => {
//                     CheckResult {
//                         result: Err(e.to_string()),
//                         process_out: None,
//                         children_results: None,
//                     }
//                 }
//             }
//         },
//     }
// }

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::predikit::data::instance::ChkInstanceBuilder;
    // use crate::predikit::data::instance::RunEnv;
    // use crate::predikit::functions::test_utils::{single_test_path_param, TestParamsBuilder};

    // #[test]
    // fn test_file_exists() {
    //     let check_fn = cd_file_exists();
    //     let ci = ChkInstanceBuilder::new(&check_fn)
    //         .with_params(single_test_path_param()).build();

    //     let check_result = ci.run_check(&RunEnv::default());
    //     assert!(check_result.result.unwrap());
    // }

    // #[test]
    // fn test_file_is_executable_no() {
    //     let check_fn = cd_file_is_executable();
    //     let ci = ChkInstanceBuilder::new(&check_fn)
    //         .with_params(single_test_path_param()).build();

    //     let check_result = ci.run_check(&RunEnv::default());
    //     assert!(!check_result.result.unwrap());
    // }

    // #[test]
    // fn test_file_is_executable_yes() {
    //     let check_fn = cd_file_is_executable();
    //     let ci = ChkInstanceBuilder::new(&check_fn)
    //         .with_params(TestParamsBuilder::new().with_string("path", "/bin/bash").build())
    //         .build();

    //     let check_result = ci.run_check(&RunEnv::default());
    //     assert!(check_result.result.unwrap());
    // }
    // //
    // // #[test]
    // // fn test_file_is_on_path() {
    // //     let run_env = RunEnv::default();
    // //     let ci = ChkInstance {
    // //         name: "test_file_is_on_path".to_owned(),
    // //         check_def: &cd_file_is_on_path(),
    // //         actual_params: param_filename("rustc"),
    // //         children: None,
    // //         negated: false,
    // //         parsed_check: None,
    // //     };
    // //
    // //     let check_result = ci.run_check(&run_env);
    // //     assert!(check_result.result.unwrap());
    // // }

    // // #[test]
    // // fn test_file_is_on_path_full() {
    // //     let run_env = RunEnv::default();
    // //     let ci = ChkInstance {
    // //         name: "test_file_is_on_path_full".to_owned(),
    // //         check_def: &cd_file_is_on_path(),
    // //         actual_params: param_filename("/bin/sh"),
    // //         children: None,
    // //         negated: false,
    // //         parsed_check: None,
    // //     };
    // //
    // //     let check_result = ci.run_check(&run_env);
    // //     assert!(check_result.result.unwrap());
    // // }

    // // #[test]
    // // fn test_file_is_on_path_no() {
    // //     let run_env = RunEnv::default();
    // //     let ci = ChkInstance {
    // //         name: "test_file_is_on_path_no".to_owned(),
    // //         check_def: &cd_file_is_on_path(),
    // //         actual_params: param_filename("asdasdlkasljkasd"),
    // //         children: None,
    // //         negated: false,
    // //         parsed_check: None,
    // //     };
    // //
    // //     let check_result = ci.run_check(&run_env);
    // //     assert_eq!(false, check_result.result.unwrap());
    // // }

    // //
    // // #[test]
    // // fn test_file_exec_yes() {
    // //     let run_env = RunEnv::default();
    // //     let ci = CheckInstance {
    // //         name: "test_file_exec_yes".to_owned(),
    // //         check_def: &cd_shell_exec(),
    // //         inputs: test_param("cmdline", "true"),
    // //         children: None,
    // //         negated: false,
    // //         parsed_check: None,
    // //     };
    // //
    // //     let check_result = ci.run_check(&run_env);
    // //     assert!(check_result.result.unwrap());
    // // }
    // //
    // //
    // // #[test]
    // // fn test_file_exec_with_stdout() {
    // //     let run_env = RunEnv::default();
    // //     let ci = CheckInstance {
    // //         name: "test_file_exec_with_stdout".to_owned(),
    // //         check_def: &cd_shell_exec(),
    // //         inputs: test_param("cmdline", "echo -n 'this is a test'"),
    // //         children: None,
    // //         negated: false,
    // //         parsed_check: None,
    // //     };
    // //
    // //     let check_result = ci.run_check(&run_env);
    // //     assert!(check_result.result.unwrap());
    // //     assert_eq!("this is a test".to_owned(), check_result.process_out.unwrap().stdout.unwrap());
    // // }
    // //
    // //
    // // #[test]
    // // fn test_file_exec_with_stderr() {
    // //     let run_env = RunEnv::default();
    // //     let ci = CheckInstance {
    // //         name: "test_file_exec_with_stderr".to_owned(),
    // //         check_def: &cd_shell_exec(),
    // //         inputs: test_param("cmdline", ">&2 echo -n 'this is a test'"),
    // //         children: None,
    // //         negated: false,
    // //         parsed_check: None,
    // //     };
    // //
    // //     let check_result = ci.run_check(&run_env);
    // //     assert!(check_result.result.unwrap());
    // //     assert_eq!("this is a test".to_owned(), check_result.process_out.unwrap().stderr.unwrap());
    // // }
    // //
    // // #[test]
    // // fn test_file_exec_no() {
    // //     let run_env = RunEnv::default();
    // //     let ci = CheckInstance {
    // //         name: "test_file_exec_no".to_owned(),
    // //         check_def: &cd_shell_exec(),
    // //         inputs: test_param("cmdline", "false"),
    // //         children: None,
    // //         negated: false,
    // //         parsed_check: None,
    // //     };
    // //
    // //     let check_result = ci.run_check(&run_env);
    // //     assert_eq!(false, check_result.result.unwrap());
    // // }
    // //
    // //
    // // #[test]
    // // fn test_file_exec_err() {
    // //     // TODO: test output etc
    // //     let run_env = RunEnv::default();
    // //     let mut params: HashMap<String, String> = HashMap::new();
    // //     params.insert("cmdline".to_owned(), "aaaaaaaa".to_owned());
    // //     params.insert("shell".to_owned(), "aaaaaaaa".to_string());
    // //     let ci = CheckInstance {
    // //         name: "test_file_exec_err".to_owned(),
    // //         check_def: &cd_shell_exec(),
    // //         // seems like an unlikely command name? I'm sure there's a better way to do this
    // //         inputs: params,
    // //         children: None,
    // //         negated: false,
    // //         parsed_check: None,
    // //     };
    // //
    // //     let check_result = ci.run_check(&run_env);
    // //     assert!(check_result.result.is_err());
    // // }
}
