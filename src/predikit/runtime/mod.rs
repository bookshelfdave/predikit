//
// fn verify_params(&self) -> Result<(), ChkResult> {
//     let required_params_count = self
//         .check_def
//         .formal_params
//         .iter()
//         .filter(|param| param.required)
//         .count();
//
//     let actual_params_count = self.inputs.len();
//
//     // check that all parameters passed in self.inputs are in the self.check_def.formal_params vec
//
//     if actual_params_count < required_params_count {
//         let msg = format!(
//             "Missing required parameters for function {}",
//             self.check_def.name
//         );
//         return Err(ChkResult {
//             result: Err(msg),
//             process_out: None,
//             children_results: None,
//         });
//     }
//
//     // Then validate no extra parameters were provided
//     for input_param in self.inputs.keys() {
//         if !self.check_def.formal_params.iter().any(|p| p.name == *input_param) {
//             println!("THIS CHECK IS BROKEN");
//             // return Err(CheckResult {
//             //     result: Err(format!("Unknown parameter provided: {}", input_param)),
//             //     process_out: None,
//             // });
//         }
//     }
//     Ok(())
// }
//
// pub fn run_check(&self, run_env: &RunEnv) -> ChkResult {
//     match self.verify_params() {
//         Err(check_err) => check_err,
//         Ok(_) => {
//             let r = (self.check_def.check_fn)(run_env, &self.inputs, self);
//             if self.negated {
//                 ChkResult {
//                     result: Ok(!r.result.unwrap()),
//                     process_out: r.process_out,
//                     children_results: r.children_results,
//                 }
//             } else {
//                 r
//             }
//         }
//     }
// }
//
// // couples a ParsedCheck + CheckDef, should probably be in a separate module
// fn inputs_to_map(parsed_check: &ParsedCheck, check_def: &ChkDef) -> HashMap<String, String> {
//     let mut inputs = HashMap::new();
//
//     let required_params: Vec<&ChkParam> = check_def.formal_params.iter().filter(|p| p.required).collect();
//
//     parsed_check.unnamed_params.iter().enumerate().for_each(|(i, param)| {
//         // print what's happening
//         debug!("setting param {}: to [{}]", i, param);
//         inputs.insert(required_params[i].name.clone(), param.clone());
//     });
//
//     for param in &parsed_check.named_params {
//         debug!("setting param {}: to [{}]", &param.name, &param.value);
//         inputs.insert(param.name.clone(), param.value.clone());
//     }
//     inputs
// }
//
// // careful of the recursion here
// pub fn instantiate_check<'a>(parsed_check: &'a ParsedCheck, reg: &'a ChkDefRegistry) -> Result<ChkInstance<'a>, String> {
//     if let Some(cd) = reg.get_checkdef(&parsed_check.fn_name) {
//         let inputs = inputs_to_map(&parsed_check, &cd);
//
//         let all_children: Vec<Result<ChkInstance, String>> = parsed_check.children.iter().map(|child| {
//             debug!("Processing child {}", &child.fn_name);
//             instantiate_check(child, reg)
//         }).collect();
//
//         if !all_children.iter().all(|res| res.is_ok()) {
//             return Err("Child check has an error".to_owned());
//         }
//         let children = all_children.into_iter().map(|c| c.unwrap()).collect();
//         Ok(ChkInstance {
//             name: parsed_check.name.clone(),
//             check_def: cd,
//             inputs,
//             children: Some(children),
//             negated: parsed_check.negated,
//             parsed_check: Some(parsed_check),
//         })
//     } else {
//         Err(format!("Check definition {} not found", parsed_check.fn_name))
//     }
// }
//
//
// pub fn process_check_defs(parsed_check_file: ParsedCheckFile) {
//     let reg = ChkDefRegistry::new_with_builtins();
//     for parsed_check in parsed_check_file.checks {
//         debug!("Check: {:?}", &parsed_check);
//
//         let instance_tree = instantiate_check(&parsed_check, &reg);
//         if let Err(err) = instance_tree {
//             panic!("{}", err);
//         }
//         let instance_tree = instance_tree.unwrap();
//
//         let run_env = RunEnv::default();
//
//         let run_result = instance_tree.run_check(&run_env);
//
//         debug!("\n\nrun_result: {:?}", &run_result);
//         //print!("Check: {} @ line {}: ", parsed_check.name, parsed_check.check_line);
//         render(&instance_tree, &run_result, 0);
//     }
// }
//
// pub fn render(check_instance: &ChkInstance, check_result: &ChkResult, prefix_ws: u8) {
//     use colored::Colorize;
//     // TODO: put ParsedCheck inside of CheckInstance
//     let prefix_ws_str = " ".repeat(prefix_ws as usize);
//     let first_param = check_instance.check_def.formal_params.first().unwrap();
//     let first_param_value = check_instance.inputs.get(first_param.name.as_str()).unwrap();
//     print!("{}> [{}] {} {}: ", prefix_ws_str, check_instance.name, check_instance.check_def.name, first_param_value);
//     match &check_result.result {
//         Ok(true) => println!("{}", "OK".green()),
//         Ok(false) => println!("{}", "FAIL".red()),
//         Err(err_msg) => println!("{}", "ERROR".red()),
//     };
//     if let Some(children) = &check_instance.children {
//         for child in children {
//             render(&child, check_result, prefix_ws + 4);
//         }
//     }
// }
