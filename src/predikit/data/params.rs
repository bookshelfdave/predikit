// Copyright (c) 2025 Dave Parfitt

use super::instance::ContentAddress;
use super::ParsedDuration;
use crate::predikit::data::ChkParamType;
use std::borrow::Cow;
use std::env;
use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum NamedType {
    PtnString,
    PtnInt,
    PtnBool,
    PtnDuration,
    PtnPath,
}

impl fmt::Display for NamedType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NamedType::PtnString => write!(f, "String"),
            NamedType::PtnInt => write!(f, "Int"),
            NamedType::PtnBool => write!(f, "Bool"),
            NamedType::PtnDuration => write!(f, "Duration"),
            NamedType::PtnPath => write!(f, "Path"),
        }
    }
}

impl From<&NamedType> for ChkParamType {
    fn from(ty: &NamedType) -> ChkParamType {
        match *ty {
            NamedType::PtnString => ChkParamType::PkString,
            NamedType::PtnInt => ChkParamType::PkInt,
            NamedType::PtnBool => ChkParamType::PkBool,
            NamedType::PtnDuration => ChkParamType::PkDuration,
            NamedType::PtnPath => ChkParamType::PkPath,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChkParamInternalValue {
    PkString(String),
    PkInt(i64),
    PkBool(bool),
    PkTypeName(NamedType),
    // duration has it's own type because it carries the original unparsed value, and the conversion
    // to a Duration
    PkDuration(ParsedDuration),

    // path has it's own type because environment vars can be referenced in a path _at runtime_
    PkPath(String),
}

pub type ChkActualParams = HashMap<String, ChkActualParam>;

#[derive(Debug, Clone)]
pub struct ChkActualParam {
    pub name: String,
    pub value: ChkParamInternalValue,
    pub content_address: ContentAddress,
}

impl ChkActualParam {
    pub fn new_string(name: String, s: String, content_address: ContentAddress) -> Self {
        ChkActualParam {
            name,
            value: ChkParamInternalValue::PkString(s),
            content_address,
        }
    }

    pub fn new_int(name: String, i: i64, content_address: ContentAddress) -> Self {
        ChkActualParam {
            name,
            value: ChkParamInternalValue::PkInt(i),
            content_address,
        }
    }

    pub fn new_bool(name: String, b: bool, content_address: ContentAddress) -> Self {
        ChkActualParam {
            name,
            value: ChkParamInternalValue::PkBool(b),
            content_address,
        }
    }

    pub fn new_duration(name: String, d: ParsedDuration, content_address: ContentAddress) -> Self {
        ChkActualParam {
            name,
            value: ChkParamInternalValue::PkDuration(d),
            content_address,
        }
    }

    pub fn new_path(name: String, s: String, content_address: ContentAddress) -> Self {
        ChkActualParam {
            name,
            value: ChkParamInternalValue::PkPath(s),
            content_address,
        }
    }

    pub fn new_named_type(
        name: String,
        type_name: NamedType,
        content_address: ContentAddress,
    ) -> Self {
        ChkActualParam {
            name,
            value: ChkParamInternalValue::PkTypeName(type_name),
            content_address,
        }
    }

    // Not to be confused with NamedType
    pub fn type_name(&self) -> String {
        match &self.value {
            ChkParamInternalValue::PkString(_) => "String".to_owned(),
            ChkParamInternalValue::PkInt(_) => "Int".to_owned(),
            ChkParamInternalValue::PkBool(_) => "Bool".to_owned(),
            ChkParamInternalValue::PkTypeName(param_type_name) => {
                format!("Typename({:?})", param_type_name).to_owned()
            }
            ChkParamInternalValue::PkDuration(_) => "Duration".to_owned(),
            ChkParamInternalValue::PkPath(_) => "Path".to_owned(),
        }
    }

    pub fn is_type(&self, t: &ChkParamType) -> bool {
        match &self.value {
            ChkParamInternalValue::PkString(_) => *t == ChkParamType::PkString,
            ChkParamInternalValue::PkInt(_) => *t == ChkParamType::PkInt,
            ChkParamInternalValue::PkBool(_) => *t == ChkParamType::PkBool,
            ChkParamInternalValue::PkTypeName(_) => *t == ChkParamType::PkTypeName,
            ChkParamInternalValue::PkDuration(_) => *t == ChkParamType::PkDuration,
            ChkParamInternalValue::PkPath(_) => *t == ChkParamType::PkPath,
        }
    }

    pub fn is_coercible_to_path(&self) -> bool {
        match &self.value {
            ChkParamInternalValue::PkString(_) => true,
            ChkParamInternalValue::PkPath(_) => true,
            _ => false,
        }
    }

    // NOTE: this is matching named type names!
    pub fn is_named_type(&self, nt: &NamedType) -> bool {
        match &self.value {
            ChkParamInternalValue::PkTypeName(ntt) => nt == ntt,
            _ => panic!("Not a named type"),
        }
    }

    pub fn get_string(&self) -> &str {
        if let ChkParamInternalValue::PkString(s) = &self.value {
            s
        } else {
            panic!("Expected a string, got a {}", self.type_name())
        }
    }

    pub fn get_raw_path(&self) -> &str {
        if let ChkParamInternalValue::PkPath(s) = &self.value {
            s
        } else {
            panic!("Expected a path, got a {}", self.type_name())
        }
    }

    pub fn get_path(&self) -> Result<String, String> {
        if let ChkParamInternalValue::PkPath(p) = &self.value {
            match shellexpand::env_with_context(p, shellexpand_context) {
                Ok(s) => Ok(s.to_string()),
                Err(e) => {
                    let msg = format!("Error expanding path: {:#?}", e.to_string());
                    Err(msg)
                }
            }
        } else if let ChkParamInternalValue::PkString(s) = &self.value {
            Ok(s.clone())
        } else {
            panic!("Expected a path or a string, got a {}", self.type_name())
        }
    }

    pub fn get_int(&self) -> i64 {
        if let ChkParamInternalValue::PkInt(i) = &self.value {
            *i
        } else {
            panic!("Expected an int, got a {}", self.type_name())
        }
    }

    pub fn get_bool(&self) -> bool {
        if let ChkParamInternalValue::PkBool(b) = &self.value {
            *b
        } else {
            panic!("Expected a bool, got a {}", self.type_name())
        }
    }

    pub fn get_duration(&self) -> ParsedDuration {
        if let ChkParamInternalValue::PkDuration(d) = &self.value {
            d.clone()
        } else {
            panic!("Expected a duration, got a {}", self.type_name())
        }
    }

    pub fn get_named_type(&self) -> &NamedType {
        if let ChkParamInternalValue::PkTypeName(tn) = &self.value {
            tn
        } else {
            panic!("Expected a named type, got a {}", self.type_name())
        }
    }

    pub fn value_as_string(&self) -> String {
        match &self.value {
            ChkParamInternalValue::PkString(s) => s.to_string(),
            ChkParamInternalValue::PkInt(i) => i.to_string(),
            ChkParamInternalValue::PkBool(b) => b.to_string(),
            ChkParamInternalValue::PkDuration(d) => d.to_string(),
            ChkParamInternalValue::PkPath(p) => p.to_string(),
            ChkParamInternalValue::PkTypeName(param_type_name) => {
                format!("{:?}", param_type_name).to_string()
            }
        }
    }
}

impl ChkParamInternalValue {
    pub fn is_type(&self, t: ChkParamType) -> bool {
        match self {
            ChkParamInternalValue::PkString(_) => t == ChkParamType::PkString,
            ChkParamInternalValue::PkInt(_) => t == ChkParamType::PkInt,
            ChkParamInternalValue::PkBool(_) => t == ChkParamType::PkBool,
            ChkParamInternalValue::PkTypeName(_) => t == ChkParamType::PkTypeName,
            ChkParamInternalValue::PkDuration(_) => t == ChkParamType::PkDuration,
            ChkParamInternalValue::PkPath(_) => t == ChkParamType::PkPath,
        }
    }

    pub fn get_string(&self) -> String {
        if let ChkParamInternalValue::PkString(s) = &self {
            s.to_owned()
        } else {
            panic!("Expected a string");
        }
    }

    pub fn get_int(&self) -> i64 {
        if let ChkParamInternalValue::PkInt(i) = &self {
            *i
        } else {
            panic!("Expected an int");
        }
    }

    pub fn get_bool(&self) -> bool {
        if let ChkParamInternalValue::PkBool(b) = &self {
            *b
        } else {
            panic!("Expected a bool");
        }
    }

    pub fn get_duration(&self) -> ParsedDuration {
        if let ChkParamInternalValue::PkDuration(d) = &self {
            d.clone()
        } else {
            panic!("Expected a duration");
        }
    }

    pub fn get_path(&self) -> String {
        if let ChkParamInternalValue::PkPath(s) = &self {
            s.to_owned()
        } else {
            panic!("Expected a path");
        }
    }
}

impl fmt::Display for ChkParamInternalValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn shellexpand_context(s: &str) -> Result<Option<Cow<'static, str>>, env::VarError> {
    match env::var(s) {
        Ok(value) => Ok(Some(value.into())),
        Err(env::VarError::NotPresent) => Ok(Some("".into())),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn foo() {}
}
