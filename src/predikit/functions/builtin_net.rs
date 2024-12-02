// Copyright (c) 2025 Dave Parfitt

use crate::predikit::data::instance::ChkResult;
use crate::predikit::data::params::ChkActualParams;
use crate::predikit::data::{ChkDef, ChkParamType, FParamsBuilder};
use port_scanner::{scan_port, scan_port_addr};

pub fn cd_port_open() -> ChkDef {
    ChkDef {
        name: "port_open?".to_owned(),
        is_group: false,
        accepts_children: false,
        template_params: None,
        is_query: false,
        formal_params: FParamsBuilder::new()
            .add_param("port", ChkParamType::PkInt)
            .required()
            .finish_param()
            .build(),
        check_fn: |_, params: &ChkActualParams, _| -> ChkResult {
            let p0 = params.get("port").unwrap();
            let port = p0.get_int();
            if !(0..=65535).contains(&port) {
                return ChkResult {
                    result: Err("port must be >= 0 && <= 65535)".to_string()),
                    process_out: None,
                    children_results: None,
                };
            }
            let v = scan_port(port as u16);
            ChkResult {
                result: Ok(v),
                process_out: None,
                children_results: None,
            }
        },
    }
}

pub fn cd_port_addr_open() -> ChkDef {
    ChkDef {
        name: "port_addr_open?".to_owned(),
        is_group: false,
        accepts_children: false,
        template_params: None,
        is_query: false,
        formal_params: FParamsBuilder::new()
            .add_param("addr_port", ChkParamType::PkString)
            .required()
            .finish_param()
            .build(),
        check_fn: |_, params: &ChkActualParams, _| -> ChkResult {
            let p0 = params.get("addr_port").unwrap();
            let addr_port = p0.get_string();
            let v = scan_port_addr(addr_port);
            ChkResult {
                result: Ok(v),
                process_out: None,
                children_results: None,
            }
        },
    }
}

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
