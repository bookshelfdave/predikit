// Copyright (c) 2025 Dave Parfitt

pub mod builtin;
pub mod builtin_fs;
pub mod builtin_net;
pub mod builtin_tools;
pub mod waiting;

#[cfg(test)]
pub mod test_utils {
    // use crate::predikit::data::instance::{ChkActualParams, ContentAddress};
    // use crate::predikit::data::params::ChkActualParam;
    // use std::collections::HashMap;

    // pub struct TestParamsBuilder {
    //     params: HashMap<String, ChkActualParam>,
    // }

    // impl Default for TestParamsBuilder {
    //     fn default() -> Self {
    //         TestParamsBuilder::new()
    //     }
    // }

    // impl TestParamsBuilder {
    //     pub fn new() -> Self {
    //         TestParamsBuilder {
    //             params: HashMap::new(),
    //         }
    //     }

    //     pub fn with_string(mut self, name: &str, value: &str) -> Self {
    //         self.params.insert(
    //             name.to_owned(),
    //             ChkActualParam::new_string(&value.to_owned(), ContentAddress::default()),
    //         );
    //         self
    //     }

    //     pub fn with_this_file_as_path(mut self) -> Self {
    //         let f = file!();
    //         self.params.insert(
    //             "path".to_owned(),
    //             ChkActualParam::new_string(&f.to_owned(), ContentAddress::default()),
    //         );
    //         self
    //     }

    //     pub fn with_int(mut self, name: &str, value: i64) -> Self {
    //         self.params.insert(
    //             name.to_owned(),
    //             ChkActualParam::new_int(value, ContentAddress::default()),
    //         );
    //         self
    //     }

    //     pub fn with_bool(mut self, name: &str, value: bool) -> Self {
    //         self.params.insert(
    //             name.to_owned(),
    //             ChkActualParam::new_bool(value, ContentAddress::default()),
    //         );
    //         self
    //     }

    //     pub fn build(self) -> ChkActualParams {
    //         self.params
    //     }
    // }

    // pub fn single_test_path_param() -> ChkActualParams {
    //     TestParamsBuilder::new().with_this_file_as_path().build()
    // }
}
