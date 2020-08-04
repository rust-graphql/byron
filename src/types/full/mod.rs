pub mod enums;
pub mod inputs;
pub mod interfaces;
pub mod objects;
pub mod scalars;
pub mod unions;

use super::*;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum Type {
    #[serde(rename = "INPUT_OBJECT")]
    Input(inputs::Type),

    #[serde(rename = "OBJECT")]
    Object(objects::Type),

    #[serde(rename = "ENUM")]
    Enum(enums::Type),

    #[serde(rename = "INTERFACE")]
    Interface(interfaces::Type),

    #[serde(rename = "UNION")]
    Union(unions::Type),

    #[serde(rename = "SCALAR")]
    Scalar(scalars::Type),
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Type::Input(t) => t.to_tokens(tokens),
            Type::Object(t) => t.to_tokens(tokens),
            Type::Enum(t) => t.to_tokens(tokens),
            Type::Interface(t) => t.to_tokens(tokens),
            Type::Union(t) => t.to_tokens(tokens),
            Type::Scalar(t) => t.to_tokens(tokens),
        }
    }
}
