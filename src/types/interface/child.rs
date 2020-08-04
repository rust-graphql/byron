use super::*;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum Type {
    #[serde(rename = "OBJECT")]
    Object { name: String },

    #[serde(rename = "INTERFACE")]
    Interface { name: String },
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Type::Interface { name } => name,
            Type::Object { name } => name,
        };

        let pascal = name.to_pascal_case();
        let pascal = Ident::new(&pascal, Span::call_site());

        tokens.extend(quote! {
            #[serde(rename = #name)]
            #pascal(#pascal)
        })
    }
}
