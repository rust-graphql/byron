use super::*;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum Type {
    #[serde(rename = "INPUT_OBJECT")]
    Input { name: String },

    #[serde(rename = "OBJECT")]
    Object { name: String },

    #[serde(rename = "ENUM")]
    Enum { name: String },

    #[serde(rename = "INTERFACE")]
    Interface { name: String },

    #[serde(rename = "UNION")]
    Union { name: String },

    #[serde(rename = "SCALAR")]
    Scalar { name: String },
}

impl Type {
    pub fn orig(&self) -> &str {
        match self {
            Type::Input { name } => name,
            Type::Object { name } => name,
            Type::Enum { name } => name,
            Type::Interface { name } => name,
            Type::Union { name } => name,
            Type::Scalar { name } => name,
        }
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pascal = self.orig().to_snake_case().to_pascal_case(); // Normalize
        let pascal = Ident::new(&pascal, Span::call_site());
        tokens.extend(quote! { #pascal })
    }
}
