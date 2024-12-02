pub mod builtin;
pub mod waiting;
pub mod builtin_net;
pub mod builtin_fs;

#[cfg(test)]
pub mod test_utils {
    use crate::predikit::data::{ChkActualParams, ChkParamInstance};
    use std::collections::HashMap;

    pub struct TestParamsBuilder {
        params: HashMap<String, ChkParamInstance>,
    }


    impl TestParamsBuilder {
        pub fn new() -> Self {
            TestParamsBuilder {
                params: HashMap::new()
            }
        }

        pub fn with_string(mut self, name: &str, value: &str) -> Self {
            self.params.insert(name.to_owned(), ChkParamInstance::PkString(value.to_owned()));
            self
        }

        pub fn with_this_file_as_path(mut self) -> Self {
            let f = file!();
            self.params.insert("path".to_owned(), ChkParamInstance::PkString(f.to_owned()));
            self
        }

        pub fn with_int(mut self, name: &str, value: i64) -> Self {
            self.params.insert(name.to_owned(), ChkParamInstance::PkInt(value));
            self
        }

        pub fn with_bool(mut self, name: &str, value: bool) -> Self {
            self.params.insert(name.to_owned(), ChkParamInstance::PkBool(value));
            self
        }

        pub fn build(self) -> ChkActualParams {
            self.params
        }
    }

    pub fn single_test_path_param() -> ChkActualParams {
        TestParamsBuilder::new().with_this_file_as_path().build()
    }
}


