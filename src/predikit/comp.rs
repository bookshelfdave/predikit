// Copyright (c) 2025 Dave Parfitt

use super::data::instance::{ChkInstance, ContentAddress};

pub mod ast;
pub mod compiler;
pub mod errors;
pub mod tokens;
pub mod validators;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub pkparser, "/predikit/comp/pkparser.rs");

#[derive(Debug, Clone, PartialEq)]
pub enum CompilerErrorType {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub filename: Option<String>,
    pub content_address: ContentAddress,
    pub message: String,
    pub error_type: CompilerErrorType,
}

#[derive(Debug, Clone)]
pub struct CompiledCheckFileOut<'chkdef> {
    pub next_id: usize,
    pub filename: Option<String>, // or Path?
    pub errors: Vec<CompileError>,
    pub instances: Vec<ChkInstance<'chkdef>>,
}

impl<'a> CompiledCheckFileOut<'a> {
    pub fn new(filename: Option<String>) -> Self {
        CompiledCheckFileOut {
            next_id: 1000,
            errors: vec![],
            filename,
            instances: vec![],
        }
    }

    pub fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_error(
        &mut self,
        filename: Option<String>,
        position: std::ops::Range<usize>,
        message: String,
    ) {
        let content_address = position;
        self.errors.push(CompileError {
            filename,
            content_address,
            message,
            error_type: CompilerErrorType::Error,
        });
    }

    pub fn add_instance(&mut self, inst: ChkInstance<'a>) {
        self.instances.push(inst);
    }
}

#[cfg(test)]
mod tests {

    use crate::predikit::{comp::pkparser, data::ChkParamType};

    #[test]
    fn test_param_int() {
        let param_int = pkparser::ActualParamParser::new()
            .parse("someint: 100")
            .unwrap();
        assert!(param_int.is_type(&ChkParamType::PkInt));
        assert_eq!(100, param_int.get_int());
    }

    #[test]
    fn test_param_string() {
        let param_string = pkparser::ActualParamParser::new()
            .parse("somestring: \"foo\"")
            .unwrap();
        assert!(param_string.is_type(&ChkParamType::PkString));
        assert_eq!("foo".to_string(), param_string.get_string());

        let param_string = pkparser::ActualParamParser::new()
            .parse("somestring: \"\"")
            .unwrap();
        assert!(param_string.is_type(&ChkParamType::PkString));
        assert_eq!("".to_string(), param_string.get_string());
    }

    #[test]
    fn test_param_bool() {
        let param_bool = pkparser::ActualParamParser::new()
            .parse("somebool: true")
            .unwrap();
        assert!(param_bool.is_type(&ChkParamType::PkBool));
        assert_eq!(true, param_bool.get_bool());
    }

    #[test]
    fn test_actual_params() {
        let aps = pkparser::ActualParamsParser::new()
            .parse("somebool: true someint: 100 somestring: \"foo bar 123\"")
            .unwrap();
        assert_eq!(aps.len(), 3);
        let somebool = aps.get("somebool").unwrap();
        assert!(somebool.get_bool());

        let someint = aps.get("someint").unwrap();
        assert_eq!(100, someint.get_int());

        let somestring = aps.get("somestring").unwrap();
        assert_eq!("foo bar 123".to_string(), somestring.get_string());
    }

    #[test]
    fn test_check_def() {
        let cd = pkparser::CheckDefParser::new()
            .parse("  @test not exists? { path: \"/home/foo\" }")
            .unwrap();
        assert!(cd.is_negated);
        assert!(cd.is_retrying);
        assert_eq!("exists?".to_string(), cd.fn_name);
        let path = cd.actual_params.get("path").unwrap();
        assert_eq!("/home/foo".to_string(), path.get_string());
        assert_eq!(1, cd.actual_params.keys().len());
    }

    #[test]
    fn test_check_def_error() {
        // double { should fail
        let e =
            pkparser::CheckDefParser::new().parse("  @test not exists? {{ path: \"/home/foo\" }");
        let urt = e.unwrap_err();
        if let lalrpop_util::ParseError::UnrecognizedToken {
            token, expected: _, ..
        } = urt
        {
            assert_eq!(token.0, 21);
            assert_eq!(token.1, lalrpop_util::lexer::Token(16, "{"));
            assert_eq!(token.2, 22);
        } else {
            panic!("Should have failed");
        }
    }

    #[test]
    fn test_check_nested() {
        let s = r#"
            all {
               title: "some group test"
                any {
                    title: "some test 1"
                    test not exists1? {
                        path: "/home/dparfitt/foo"
                    }
                    @test not exists2? {
                        path: "/home/dparfitt/bar"
                    }
                }

                none {
                    title: "some test 2"
                    test exists3? {
                        path: "/home/dparfitt/x"
                    }
                    @test exists4? {
                        path: "/home/dparfitt/y"
                    }
                }
            }
            "#;
        let cd = pkparser::GroupOrCheckParser::new().parse(s).unwrap();
        println!("{:#?}", cd);
        assert_eq!("all".to_string(), cd.fn_name);
        assert!(!cd.is_negated);
        assert!(!cd.is_retrying);
        assert_eq!(1, cd.actual_params.keys().len());
        assert_eq!(
            "some group test".to_string(),
            cd.actual_params.get("title").unwrap().get_string()
        );
        assert_eq!(2, cd.children.len());

        // Any block
        let any = cd.children.first().unwrap();
        assert_eq!(2, any.children.len());
        assert!(!any.is_negated);
        assert!(!any.is_retrying);
        assert_eq!(1, any.actual_params.keys().len());
        assert_eq!(
            "some test 1".to_string(),
            any.actual_params.get("title").unwrap().get_string()
        );
        assert_eq!("any".to_string(), any.fn_name);
        // first test in any
        let exists1 = any.children.first().unwrap();
        assert!(exists1.is_negated);
        assert!(!exists1.is_retrying);
        assert!(!exists1.is_group);

        // second test in any
        let exists2 = any.children.get(1).unwrap();
        assert!(exists2.is_retrying);

        // None block
        let none = cd.children.get(1).unwrap();
        assert_eq!(2, none.children.len());

        assert!(!none.is_negated);
        assert!(!none.is_retrying);
        assert_eq!(1, none.actual_params.keys().len());
        assert_eq!(
            "some test 2".to_string(),
            none.actual_params.get("title").unwrap().get_string()
        );
        assert_eq!("none".to_string(), none.fn_name);
    }

    // #[test]
    // fn test_toplevel() {
    //     let lexer = Lexer::new(
    //         &r#"
    //            tool free_space_gt? {
    //                cmd_template: "df -Ph {{path}} | tr -d '%' | awk 'NR==2{ exit $5>{{percentage}} ? 0 : 1}'"
    //                 $path {
    //                    type: String
    //                    required: true
    //                }
    //                $percentage {
    //                    type: Int
    //                    required: true
    //                }
    //                $stuff {
    //                    type: Bool
    //                    required: true
    //                }
    //            }
    //            all {
    //                title: "Testing core commands"
    //                test free_space_gt? {
    //                    path: "/home/dparfitt"
    //                    percentage: 50
    //                    stuff: true
    //                }
    //            }
    //            "#,
    //     );
    //     let cd = pkparser::TopLevelParser::new().parse(lexer).unwrap();
    //     println!("{:#?}", cd);
    // }
}
