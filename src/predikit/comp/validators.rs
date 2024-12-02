// Copyright (c) 2025 Dave Parfitt

use crate::predikit::data::params::ChkActualParam;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use url::Url;

use super::tokens::{parse_duration_str, parse_path_str, strip_parens_and_trim, LexicalError};

pub fn parse_validator(
    name: String,
    fxn: &str,
    value: String,
    start: usize,
    end: usize,
) -> Result<ChkActualParam, LexicalError> {
    let stripped = strip_parens_and_trim(&value);

    match fxn {
        "d" | "duration" => {
            if let Some(d) = parse_duration_str(&stripped) {
                Ok(ChkActualParam::new_duration(name, d, start..end))
            } else {
                Err(LexicalError::InvalidDuration(value, start..end))
            }
        }
        "p" | "path" => {
            if let Some(p) = parse_path_str(&stripped) {
                Ok(ChkActualParam::new_path(name, p, start..end))
            } else {
                Err(LexicalError::InvalidPath(value, start..end))
            }
        }
        "perms" | "fileperms" | "filepermissions" => {
            if !stripped.chars().all(|x| x.is_digit(8)) {
                return Err(LexicalError::InvalidConversion(
                    "filepermissions".to_string(),
                    "Invalid octal value".to_string(),
                    start..end,
                ));
            }
            Ok(ChkActualParam::new_string(name, stripped, start..end))
        }
        "url" => match Url::parse(&stripped) {
            Ok(_) => Ok(ChkActualParam::new_string(name, stripped, start..end)),
            Err(e) => Err(LexicalError::InvalidConversion(
                "url".to_string(),
                e.to_string(),
                start..end,
            )),
        },

        "ip" => match stripped.parse::<IpAddr>() {
            Ok(_) => Ok(ChkActualParam::new_string(name, stripped, start..end)),
            Err(e) => Err(LexicalError::InvalidConversion(
                "ip".to_string(),
                e.to_string(),
                start..end,
            )),
        },
        "ipv4" => match stripped.parse::<Ipv4Addr>() {
            Ok(_) => Ok(ChkActualParam::new_string(name, stripped, start..end)),
            Err(e) => Err(LexicalError::InvalidConversion(
                "ipv4".to_string(),
                e.to_string(),
                start..end,
            )),
        },
        "ipv6" => match stripped.parse::<Ipv6Addr>() {
            Ok(_) => Ok(ChkActualParam::new_string(name, stripped, start..end)),
            Err(e) => Err(LexicalError::InvalidConversion(
                "ipv6".to_string(),
                e.to_string(),
                start..end,
            )),
        },
        "port" => match stripped.parse::<u16>() {
            Ok(_) => Ok(ChkActualParam::new_string(name, stripped, start..end)),
            Err(e) => Err(LexicalError::InvalidConversion(
                "port".to_string(),
                e.to_string(),
                start..end,
            )),
        },
        _ => Err(LexicalError::InvalidToken(start..end)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_parse() {
        let result = parse_validator("test".to_string(), "d", "(  10s  )".to_string(), 0, 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_duration() {
        let result = parse_validator("test".to_string(), "d", "(invalid)".to_string(), 0, 7);
        assert!(result.is_err());
    }

    #[test]
    fn test_path_parse() {
        let result = parse_validator("test".to_string(), "p", "(  /tmp/test  )".to_string(), 0, 9);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_path() {
        let result = parse_validator("test".to_string(), "p", "()".to_string(), 0, 13);
        assert!(result.is_err());
    }

    #[test]
    fn test_url_parse() {
        let result = parse_validator(
            "test".to_string(),
            "url",
            "(  http://example.com  )".to_string(),
            0,
            18,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_url() {
        let result = parse_validator("test".to_string(), "url", "(not a url)".to_string(), 0, 9);
        assert!(result.is_err());
    }

    #[test]
    fn test_ip_parse() {
        let result = parse_validator(
            "test".to_string(),
            "ip",
            "(  127.0.0.1  )".to_string(),
            0,
            9,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_ip() {
        let result = parse_validator("test".to_string(), "ip", "(not.an.ip)".to_string(), 0, 9);
        assert!(result.is_err());
    }

    #[test]
    fn test_ipv4_parse() {
        let result = parse_validator(
            "test".to_string(),
            "ipv4",
            "(  192.168.1.1  )".to_string(),
            0,
            11,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_ipv4() {
        let result = parse_validator(
            "test".to_string(),
            "ipv4",
            "(  256.1.2.3  )".to_string(),
            0,
            9,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_ipv6_parse() {
        let result = parse_validator("test".to_string(), "ipv6", "(::1)".to_string(), 0, 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_ipv6() {
        let result = parse_validator(
            "test".to_string(),
            "ipv6",
            "(invalid_ipv6)".to_string(),
            0,
            11,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_port_parse() {
        let result = parse_validator("test".to_string(), "port", "(  8080 )".to_string(), 0, 4);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_port() {
        let result = parse_validator("test".to_string(), "port", "(  65536  )".to_string(), 0, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_port2() {
        let result = parse_validator("test".to_string(), "port", "(  pants )".to_string(), 0, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_fileperms_parse() {
        let result = parse_validator(
            "test".to_string(),
            "fileperms",
            "(  644  )".to_string(),
            0,
            3,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_fileperms() {
        let result = parse_validator("test".to_string(), "fileperms", "(999)".to_string(), 0, 3);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fileperms2() {
        let result = parse_validator(
            "test".to_string(),
            "fileperms",
            "(  pants )".to_string(),
            0,
            3,
        );
        assert!(result.is_err());
    }
}
