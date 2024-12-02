// Copyright (c) 2025 Dave Parfitt

use crate::predikit::data::instance::{ChkProcessOut, ChkResult};
use crate::predikit::data::params::ChkActualParams;
use crate::predikit::data::tools::ToolDef;
use crate::predikit::data::{formal_params_to_map, ChkDef};
use handlebars::Handlebars;
use log::debug;
use std::collections::HashMap;

// this needs all the refactoring
pub fn metadef_tool(td: &ToolDef) -> ChkDef {
    ChkDef {
        name: td.tool_name.clone(),
        is_group: false,
        accepts_children: false,
        formal_params: formal_params_to_map(&td.instance_params),
        template_params: Some(td.template_params.clone()),
        is_query: false,
        check_fn: |_, params: &ChkActualParams, inst| -> ChkResult {
            debug!("params: {:#?}", params);

            // note: missing required params will fail before this function gets called... mostly.

            // template params are stored in the ChkDef
            let cmd_template = inst
                .fn_def
                .template_params
                .as_ref()
                .unwrap()
                .get("cmd_template")
                .unwrap();

            let hb = Handlebars::new();
            let mut data = HashMap::new();
            for (k, v) in params.iter() {
                data.insert(k.clone(), v.value_as_string());
            }
            let rendered_cmd = hb.render_template(cmd_template.get_string(), &data);
            if rendered_cmd.is_err() {
                return ChkResult {
                    result: Err(format!(
                        "Error rendering template: {}",
                        rendered_cmd.err().unwrap()
                    )),
                    process_out: None,
                    children_results: None,
                };
            }
            let rendered_cmd = rendered_cmd.unwrap();
            debug!("cmd_template: {}", cmd_template.get_string());
            debug!("rendered: [{}]", rendered_cmd);

            let cmd_result = std::process::Command::new("sh")
                .arg("-c")
                .arg(rendered_cmd)
                .output();

            match cmd_result {
                Ok(output) => {
                    //println!("SHELL RESULT = {}", &output.status.code().unwrap());
                    //debug!("SHELL RESULT 2= {}", &output.status.success());
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
