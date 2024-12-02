use crate::predikit::data::instance::ChkInstance;
use crate::predikit::data::{ChkActualParams, ChkDef, ChkResult, RunEnv};
use crate::predikit::functions::builtin_fs::cd_file_exists;
use log::debug;

pub fn define_builtins() -> Vec<ChkDef> {
    vec![
        cd_file_exists(),
        //cd_file_is_on_path(),
        //cd_file_is_executable(),
        //cd_shell_exec(),
        //cd_port_open(),
        //cd_port_addr_open(),
    ]
}


pub fn define_aggs() -> Vec<ChkDef> {
    vec![
        cd_all(),
        cd_any(),
        cd_none(),
    ]
}

fn agg_all(f: &Vec<ChkResult>) -> bool {
    f.iter().all(|x| *x.result.as_ref().unwrap())
}

fn agg_any(f: &Vec<ChkResult>) -> bool {
    f.iter().any(|x| *x.result.as_ref().unwrap())
}

fn agg_none(f: &Vec<ChkResult>) -> bool {
    f.iter().all(|x| !*x.result.as_ref().unwrap())
}

enum AggType {
    All,
    Any,
    None,
}


fn agg_exec(agg_type: AggType, run_env: &RunEnv, _: &ChkActualParams, this: &ChkInstance) -> ChkResult {
    let mut child_results: Vec<ChkResult> = vec![];

    for child in &this.children {
        debug!("RUNNING CHILD {} (negated? {})", &child.check_def.name, &child.negated);
        let child_result = child.run_check(run_env);
        //child_result.result.as_ref().iter().for_each(|x| println!(" --> {}", x));
        debug!("Child result: {:#?}", &child_result.result);
        child_results.push(child_result);
    }

    if this.children.is_empty() {
        return ChkResult {
            result: Err("pred.all has no children".to_owned()),
            process_out: None,
            children_results: None,
        };
    }

    debug!(">>>>>> {:#?}", &child_results);
    let agg = match agg_type {
        AggType::All => agg_all(&child_results),
        AggType::Any => agg_any(&child_results),
        AggType::None => agg_none(&child_results),
    };
    debug!("AGG RESULT {}", agg);
    let r = ChkResult {
        result: Ok(agg),
        process_out: None, // TODO: process_out
        children_results: Some(child_results),
    };
    r
}

// // TODO: deal with output from children
pub fn cd_all() -> ChkDef {
    ChkDef {
        name: "all".to_owned(),
        formal_params: vec![],
        is_group: true,
        accepts_children: true,
        check_fn: |run_env, params: &ChkActualParams, this| -> ChkResult {
            agg_exec(AggType::All, run_env, params, this)
        },
    }
}

pub fn cd_none() -> ChkDef {
    ChkDef {
        name: "none".to_owned(),
        formal_params: vec![],
        is_group: true,
        accepts_children: true,
        check_fn: |run_env, params: &ChkActualParams, this| -> ChkResult {
            agg_exec(AggType::None, run_env, params, this)
        },
    }
}

pub fn cd_any() -> ChkDef {
    ChkDef {
        name: "any".to_owned(),
        formal_params: vec![],
        is_group: true,
        accepts_children: true,
        check_fn: |run_env, params: &ChkActualParams, this| -> ChkResult {
            agg_exec(AggType::Any, run_env, params, this)
        },
    }
}


// pub fn cd_not() -> ChkDef {
//     ChkDef {
//         name: "not".to_owned(),
//         formal_params: vec![],
//         is_group: false, // TODO: is this a group or no?
//         accepts_children: true,
//         check_fn: |run_env, _: &ChkActualParams, this| -> ChkResult {
//             if this.children.is_empty() {
//                 return ChkResult {
//                     result: Err("pred.not has no children".to_owned()),
//                     process_out: None,
//                     children_results: None,
//                 };
//             }
//
//             if this.children.len() > 1 {
//                 return ChkResult {
//                     result: Err("pred.not has more than one child".to_owned()),
//                     process_out: None,
//                     children_results: None,
//                 };
//             }
//             let child = &this.children[0];
//
//             // there's only 1 child
//             debug!(">> RUNNING CHILD {} (negated? {})", &child.check_def.name, &child.negated);
//             let child_result = child.run_check(run_env);
//             match child_result.result {
//                 Ok(v) => {
//                     debug!("pre-NEGATED CHILD RESULT {:#?}", v);
//                     ChkResult {
//                         result: Ok(!v),
//                         process_out: None,
//                         children_results: Some(vec![child_result]),
//                     }
//                 }
//                 Err(_) => {
//                     ChkResult {
//                         result: Err("Child failed".to_owned()),
//                         process_out: None,
//                         children_results: Some(vec![child_result]),
//                     }
//                 }
//             }
//         },
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    use crate::predikit::data::instance::ChkInstanceBuilder;
    use crate::predikit::data::RunEnv;
    use crate::predikit::functions::builtin_fs::cd_file_exists;
    use crate::predikit::functions::test_utils::single_test_path_param;


    // #[test]
    // fn test_not_1() {
    //     let fe = cd_file_exists();
    //     let not = cd_not();
    //
    //     let ci = ChkInstanceBuilder::new(&fe)
    //         .with_params(single_test_path_param())
    //         .build();
    //
    //     let check_result = ChkInstanceBuilder::new(&not)
    //         .add_child(ci)
    //         .build()
    //         .run_check(&RunEnv::default());
    //
    //     assert_eq!(false, check_result.result.unwrap());
    // }
    //
    //
    // #[test]
    // fn test_not_2() {
    //     let fe = cd_file_exists();
    //     let not = cd_not();
    //
    //     let ci = ChkInstanceBuilder::new(&fe)
    //         .param_string("path", "ThisFileShouldNotExist")
    //         .build();
    //
    //     let check_result = ChkInstanceBuilder::new(&not)
    //         .add_child(ci)
    //         .build()
    //         .run_check(&RunEnv::default());
    //     assert_eq!(true, check_result.result.unwrap());
    // }

    #[test]
    fn test_all_pass() {
        let fe = cd_file_exists();
        let all = cd_all();

        let ci = ChkInstanceBuilder::new(&fe)
            .with_params(single_test_path_param())
            .build();

        let check_result = ChkInstanceBuilder::new(&all)
            .add_child(ci.clone())
            .add_child(ci.clone())
            .build()
            .run_check(&RunEnv::default());

        assert_eq!(true, check_result.result.unwrap());
    }

    #[test]
    fn test_all_fail() {
        let fe = cd_file_exists();
        let all = cd_all();

        let ci = ChkInstanceBuilder::new(&fe)
            .param_string("path", "ThisFileShouldNotExist")
            .build();

        let check_result = ChkInstanceBuilder::new(&all)
            .add_child(ci.clone())
            .add_child(ci.clone())
            .build()
            .run_check(&RunEnv::default());
        assert_eq!(false, check_result.result.unwrap());
    }


    #[test]
    fn test_none_pass() {
        let fe = cd_file_exists();
        let none = cd_none();

        let ci = ChkInstanceBuilder::new(&fe)
            .param_string("path", "ThisFileShouldNotExist")
            .build();

        let check_result = ChkInstanceBuilder::new(&none)
            .add_child(ci.clone())
            .add_child(ci)
            .build()
            .run_check(&RunEnv::default());
        assert_eq!(true, check_result.result.unwrap());
    }


    #[test]
    fn test_none_fail() {
        let fe = cd_file_exists();
        let none = cd_none();

        let ci = ChkInstanceBuilder::new(&fe)
            .with_params(single_test_path_param())
            .build();


        let check_result = ChkInstanceBuilder::new(&none)
            .add_child(ci.clone())
            .add_child(ci)
            .build()
            .run_check(&RunEnv::default());

        assert_eq!(false, check_result.result.unwrap());
    }

    #[test]
    fn test_any_pass() {
        let fe = cd_file_exists();
        let any = cd_any();

        let ci1 = ChkInstanceBuilder::new(&fe)
            .param_string("path", "ThisFileShouldNotExist")
            .build();

        let ci2 = ChkInstanceBuilder::new(&fe)
            .with_params(single_test_path_param())
            .build();


        let check_result = ChkInstanceBuilder::new(&any)
            .add_child(ci1.clone())
            .add_child(ci2)
            .build()
            .run_check(&RunEnv::default());

        assert_eq!(true, check_result.result.unwrap());
    }


    #[test]
    fn test_any_fail() {
        let fe = cd_file_exists();
        let any = cd_any();

        let ci = ChkInstanceBuilder::new(&fe)
            .param_string("path", "ThisFileShouldNotExist")
            .build();

        let check_result = ChkInstanceBuilder::new(&any)
            .add_child(ci.clone())
            .add_child(ci)
            .build()
            .run_check(&RunEnv::default());

        assert_eq!(false, check_result.result.unwrap());
    }
}
