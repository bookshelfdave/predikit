// Copyright (c) 2025 Dave Parfitt

// use crate::predikit::data::{ChkInstance, ChkResult, RunEnv};
// use std::fmt;
//
// #[derive(Debug, PartialEq, Eq)]
// pub struct WaitResult {
//     check_result: ChkResult,
//     iterations: u64,
//     elapsed_time: u64,
// }
//
// impl fmt::Display for WaitResult {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self.check_result)?;
//         write!(f, "\nIterations: {}", self.iterations)?;
//         write!(f, "\nElapsed time: {}ms", self.elapsed_time)
//     }
// }
//
// pub fn wait_until_pass(
//     check_instance: ChkInstance,
//     run_env: &RunEnv,
//     timeout: u64,
//     interval: u64,
// ) -> WaitResult {
//     wait_until_predicate(
//         check_instance,
//         run_env,
//         timeout,
//         interval,
//         |result| matches!(result.result, Ok(true)),
//     )
// }
//
// #[allow(dead_code)]
// pub fn wait_until_fail(
//     check_instance: ChkInstance,
//     run_env: &RunEnv,
//     timeout: u64,
//     interval: u64,
// ) -> WaitResult {
//     wait_until_predicate(
//         check_instance,
//         run_env,
//         timeout,
//         interval,
//         |result| matches!(result.result, Ok(false)),
//     )
// }
// #[allow(dead_code)]
// pub fn wait_until_err(
//     check_instance: ChkInstance,
//     run_env: &RunEnv,
//     timeout: u64,
//     interval: u64,
// ) -> WaitResult {
//     wait_until_predicate(
//         check_instance,
//         run_env,
//         timeout,
//         interval,
//         |result| result.result.is_err(),
//     )
// }
//
// fn wait_until_predicate<F>(
//     check_instance: ChkInstance,
//     run_env: &RunEnv,
//     timeout: u64,
//     interval: u64,
//     predicate: F,
// ) -> WaitResult
// where
//     F: Fn(&ChkResult) -> bool,
// {
//     // Validate interval is not zero
//     if interval == 0 {
//         return WaitResult {
//             check_result: ChkResult {
//                 result: Err("Interval must be greater than 0".to_owned()),
//                 process_out: None,
//                 children_results: None,
//             },
//             iterations: 0,
//             elapsed_time: 0,
//         };
//     }
//
//     // Validate timeout is not zero
//     if timeout == 0 {
//         return WaitResult {
//             check_result: ChkResult {
//                 result: Err("Timeout must be greater than 0".to_owned()),
//                 process_out: None,
//                 children_results: None,
//             },
//             iterations: 0,
//             elapsed_time: 0,
//         };
//     }
//
//     if timeout < interval {
//         return WaitResult {
//             check_result: ChkResult {
//                 result: Err("Timeout must be greater than interval".to_owned()),
//                 process_out: None,
//                 children_results: None,
//             },
//             iterations: 0,
//             elapsed_time: 0,
//         };
//     }
//
//     let start = std::time::Instant::now();
//     let mut elapsed = 0;
//     let mut iterations = 0;
//
//     while elapsed < timeout {
//         iterations += 1;
//         let check_result = check_instance.run_check(run_env);
//         if predicate(&check_result) {
//             return WaitResult {
//                 check_result,
//                 iterations,
//                 elapsed_time: elapsed,
//             };
//         }
//         std::thread::sleep(std::time::Duration::from_millis(interval));
//         elapsed = start.elapsed().as_millis() as u64;
//     }
//
//     WaitResult {
//         check_result: ChkResult {
//             result: Err(format!("Timeout after {}ms", elapsed)),
//             process_out: None,
//             children_results: None,
//         },
//         iterations,
//         elapsed_time: elapsed,
//     }
// }
//
//
// // TODO: this is more like an integration test, it should probably move
// #[cfg(test)]
// mod tests {
//     use crate::predikit::functions::builtin::define_builtins;
//     use crate::predikit::data::{ChkDefRegistry, ChkInstance, RunEnv};
//     use std::collections::HashMap;
//
//     use crate::predikit::functions::waiting::wait_until_pass;
//     use async_std::task;
//     use std::time::Duration;
//
//     // TODO: This is copy/pasta from builtin.rs
//     fn test_param(param_name: &str, param_value: &str) -> HashMap<String, String> {
//         let mut params: HashMap<String, String> = HashMap::new();
//         params.insert(param_name.to_owned(), param_value.to_owned());
//         return params;
//     }
//
//     // TODO: This is copy/pasta from builtin.rs
//     fn param_filename(value: &str) -> HashMap<String, String> {
//         test_param("filename", value)
//     }
//
//     // Take a function parameter that returns no value, and execute
//     // the function after a delay in millis
//     fn run_after_delay<F>(delay_ms: u64, f: F)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//         task::spawn(async move {
//             task::sleep(Duration::from_millis(delay_ms)).await;
//             f();
//         });
//     }
//
//
//     #[test]
//     fn test_wait_until1() {
//         let run_env = RunEnv::default();
//
//         let tmp_dir = tempfile::tempdir().unwrap();
//         let tmp_path = tmp_dir.path();
//
//         let testfile = format!("{}/foobar123", tmp_path.to_string_lossy().to_string());
//
//         // TODO: make this easier to setup and use for tests
//         let mut reg = ChkDefRegistry::default();
//         let bis = define_builtins();
//         bis.into_iter()
//             .for_each(|c| reg.register_checkdef(c.name.clone(), c));
//
//         let ci = ChkInstance {
//             name: "test_wait_until1".to_owned(),
//             check_def: &reg.get_checkdef("exists?").unwrap(),
//             actual_params: param_filename(&testfile),
//             children: None,
//             negated: false,
//             parsed_check: None,
//         };
//
//         run_after_delay(1000, move || {
//             let c = format!("touch {}", testfile);
//             let out = std::process::Command::new("sh")
//                 .arg("-c")
//                 .arg(c).output().expect("Couldn't run background task");
//             let _ = String::from_utf8_lossy(&out.stdout);
//             // not sure if I care about the output
//         });
//
//         let wait_result = wait_until_pass(ci, &run_env, 3000, 100);
//         //println!("Wait result: {:?}", &wait_result);
//         assert!(wait_result.check_result.result.unwrap());
//     }
// }