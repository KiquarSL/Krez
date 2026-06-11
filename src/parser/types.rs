use super::Info;
use std::fmt;

#[derive(Debug, Clone, Default)]
pub enum Type {
    #[default]
    Unknown,
    Void(Info),
    I32(Info),
    F32(Info),
    Bool(Info),
    Str(Info),
    Custom(String, Info),
    Array(Box<Type>, Info),
    Ptr(Box<Type>, Info),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unknown => write!(f, "UNKNOWN"),
            Type::Void(_) => write!(f, "void"),
            Type::I32(_) => write!(f, "i32"),
            Type::F32(_) => write!(f, "f32"),
            Type::Bool(_) => write!(f, "bool"),
            Type::Str(_) => write!(f, "string"),
            Type::Custom(name, _) => write!(f, "{name}"),
            Type::Array(ty, _) => write!(f, "[{ty}]"),
            Type::Ptr(ty, _) => write!(f, "&{ty}"),
        }
    }
}

pub fn display_args(args: Vec<(String, Type)>) -> String {
    args.iter()
        .map(|(id, ty)| format!("{id}: {ty}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl Type {
    pub fn info(&self) -> Info {
        match self {
            Type::Str(info)
            | Type::Void(info)
            | Type::I32(info)
            | Type::F32(info)
            | Type::Bool(info)
            | Type::Array(_, info)
            | Type::Custom(_, info)
            | Type::Ptr(_, info) => info.clone(),
            Type::Unknown => Info {
                line: 0,
                offset: 0,
                len: 0,
            },
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
