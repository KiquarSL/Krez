use strum;

#[derive(Debug, Clone, strum::Display, strum::EnumString)]
pub enum Type {
    #[strum(to_string = "i32")]
    I32,
    #[strum(to_string = "f32")]
    F32,
    #[strum(to_string = "bool")]
    Bool,
    #[strum(to_string = "string")]
    Str,
}
