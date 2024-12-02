// Copyright (c) 2025 Dave Parfitt

use log::debug;
use std::num::ParseIntError;

use crate::predikit::data::{instance::ContentAddress, ParsedDuration};

#[derive(Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    InvalidType(ContentAddress),
    InvalidDuration(String, ContentAddress),
    InvalidPath(String, ContentAddress),
    InvalidToken(ContentAddress),
    InvalidConversion(String, String, ContentAddress), // (conversion type, error, content address)
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

// remove first and last double quote character
// since it's coming right from the regex, the first and last double quotes should really be there :-)
pub fn strip_quotes(s: &String) -> String {
    s.strip_prefix('"')
        .unwrap()
        .strip_suffix('"')
        .unwrap()
        .to_owned()
}

pub fn strip_parens_and_trim(s: &String) -> String {
    s.strip_prefix("(")
        .unwrap()
        .strip_suffix(')')
        .unwrap()
        .trim()
        .to_owned()
}

pub fn parse_duration_str(s: &String) -> Option<ParsedDuration> {
    match duration_str::parse(s.clone()) {
        Ok(dur) => Some(ParsedDuration::new(dur, s.clone())),
        Err(e) => {
            debug!("Duration parse error:\n{}", e);
            None
        }
    }
}

pub fn parse_path_str(s: &String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.trim().to_owned())
    }
}

#[cfg(test)]
mod test {
    use crate::predikit::comp::pkparser;

    #[test]
    fn test_str_stripping() {
        assert_eq!(
            "test".to_string(),
            super::strip_parens_and_trim(&"(test)".to_string())
        );
        assert_eq!(
            "test".to_string(),
            super::strip_quotes(&"\"test\"".to_string())
        );
    }

    #[test]
    fn test_parse_duration_str() {
        let input = "10d 9h 4m 6s".to_string();
        let result = super::parse_duration_str(&input);
        assert!(result.is_some());
        let duration = result.unwrap();
        assert_eq!(duration.duration_str, "10d 9h 4m 6s");
        // TODO check .duration value

        let invalid = "invalid duration".to_string();
        let result = super::parse_duration_str(&invalid);
        assert!(result.is_none());
    }

    #[test]
    fn test_keywords() {
        let _ = pkparser::PktToolParser::new().parse("tool").unwrap();
        let _ = pkparser::PktNotParser::new().parse("not").unwrap();
        let _ = pkparser::PktTestParser::new().parse("test").unwrap();
        let _ = pkparser::PktAllParser::new().parse("all").unwrap();
        let _ = pkparser::PktAnyParser::new().parse("any").unwrap();
        let _ = pkparser::PktNoneParser::new().parse("none").unwrap();
        let _ = pkparser::PktBoolParser::new().parse("true").unwrap();
        let _ = pkparser::PktBoolParser::new().parse("false").unwrap();
        let _ = pkparser::PktRetryingParser::new().parse("@").unwrap();
        let _ = pkparser::PktDollarParser::new().parse("$").unwrap();
        let _ = pkparser::PktBraceOpenParser::new().parse("{").unwrap();
        let _ = pkparser::PktBraceCloseParser::new().parse("}").unwrap();
    }

    #[test]
    fn test_ints() {
        let p = pkparser::PktIntParser::new().parse("123").unwrap();
        let n = pkparser::PktIntParser::new().parse("-456").unwrap();
        assert_eq!(p, 123);
        assert_eq!(n, -456);
    }

    #[test]
    fn test_string_lit() {
        let s = pkparser::PktStringParser::new()
            .parse("\"hello world\"")
            .unwrap();
        // quotes are stripped
        assert_eq!(s, "hello world".to_string());
    }

    #[test]
    fn test_conv_fn() {
        // not really great tests, but make sure convfn's can parse durations and paths
        let r = pkparser::PktConvFnParser::new()
            .parse("(10d 9h 4m 6s)")
            .unwrap();
        // parens get stripped when parsing an ActualParam
        assert_eq!(r, "(10d 9h 4m 6s)".to_string());

        let r = pkparser::PktConvFnParser::new()
            .parse("(/home/foo/bar)")
            .unwrap();
        // parens get stripped when parsing an ActualParam
        assert_eq!(r, "(/home/foo/bar)".to_string());
    }

    #[test]
    fn test_identifiers() {
        let _ = pkparser::PktIDParser::new().parse("abc").unwrap();
        let _ = pkparser::PktIDParser::new().parse("abc123").unwrap();
        let _ = pkparser::PktIDParser::new().parse("abc123?").unwrap();
        let _ = pkparser::PktIDParser::new().parse("abc123!").unwrap();
        let _ = pkparser::PktIDParser::new().parse("abc_123!").unwrap();
        let _ = pkparser::PktIDParser::new().parse("abc.123!").unwrap();

        assert!(pkparser::PktIDParser::new().parse("FOO!").is_err());
        assert!(pkparser::PktIDParser::new().parse("ast$").is_err());
        assert!(pkparser::PktIDParser::new().parse("100").is_err());
    }

    #[test]
    fn test_type_name() {
        let _ = pkparser::PktTypeNameParser::new().parse("Bool").unwrap();
        let _ = pkparser::PktTypeNameParser::new().parse("Int").unwrap();
        let _ = pkparser::PktTypeNameParser::new().parse("String").unwrap();
        assert!(pkparser::PktTypeNameParser::new().parse("string").is_err());
    }

    #[test]
    fn test_comments() {
        let _ = pkparser::TopLevelParser::new()
            .parse(
                "any /* this is a test */
            { test exists?  // another test
            { path: \"foo\" } }",
            )
            .unwrap();
    }

    #[test]
    fn test_test() {
        let _ = pkparser::CheckDefParser::new()
            .parse("test not some_test_fn? {}")
            .unwrap();
        let _ = pkparser::CheckDefParser::new()
            .parse("@test some_test_fn? {}")
            .unwrap();
    }

    #[test]
    fn test_ids() {
        let _ = pkparser::PktIDParser::new().parse("abcABC_ABC?").unwrap();
        // This should fail for invalid starting character
        assert!(pkparser::PktIDParser::new().parse("_foo?").is_err());
        // This should fail for special characters
        assert!(pkparser::PktIDParser::new().parse("abc#def").is_err());
    }
}
