use super::Info;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Type {
    Unknown,

    I32(Info),
    F32(Info),
    Bool(Info),
    Str(Info),
    Custom(String, Info),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unknown => write!(f, "UNKNOWN"),
            Type::I32(_) => write!(f, "i32"),
            Type::F32(_) => write!(f, "f32"),
            Type::Bool(_) => write!(f, "bool"),
            Type::Str(_) => write!(f, "string"),
            Type::Custom(name, _) => write!(f, "{name}"),
        }
    }
}

impl Type {
    pub fn from_str(s: &str, info: Info) -> Type {
        match s {
            "i32" => Type::I32(info),
            "f32" => Type::F32(info),
            "bool" => Type::Bool(info),
            "string" => Type::Str(info),
            custom => Type::Custom(custom.to_string(), info),
        }
    }
}
