// use crate::predikit::functions::{CheckDef, CheckParam, CheckResult};
// use port_scanner::{scan_port, scan_port_addr};
// use std::collections::HashMap;
//
// pub fn cd_port_open() -> CheckDef {
//     CheckDef {
//         short_name: "net.port_open?".to_owned(),
//         formal_params: vec![CheckParam::required("port".to_owned())],
//         check_fn: |_, params: &HashMap<String, String>, _| -> CheckResult {
//             let p0 = params.get("port").unwrap();
//             let port = p0.parse::<u16>();
//             if port.is_err() {
//                 return CheckResult {
//                     result: Err("port must be a number".to_owned()),
//                     process_out: None,
//                     children_results: None,
//                 };
//             }
//             let port = port.unwrap();
//             let v = scan_port(port);
//             CheckResult {
//                 result: Ok(v),
//                 process_out: None,
//                 children_results: None,
//             }
//         },
//     }
// }
// pub fn cd_port_addr_open() -> CheckDef {
//     CheckDef {
//         short_name: "net.port_addr_open?".to_owned(),
//         formal_params: vec![CheckParam::required("ip:port".to_owned())],
//         check_fn: |_, params: &HashMap<String, String>, _| -> CheckResult {
//             let p0 = params.get("ip:port").unwrap();
//             let v = scan_port_addr(p0);
//             CheckResult {
//                 result: Ok(v),
//                 process_out: None,
//                 children_results: None,
//             }
//         },
//     }
// }
