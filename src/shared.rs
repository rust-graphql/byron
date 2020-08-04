use super::*;

#[derive(Deserialize, Debug, PartialEq, Default)]
pub struct Documentation(Option<String>);

impl ToTokens for Documentation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self.0.as_ref() {
            Some(description) => quote! { #[doc=#description] },
            None => quote! {},
        })
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct TypeName(String);

impl From<String> for TypeName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for TypeName {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl ToTokens for TypeName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pascal = self.0.to_pascal_case();
        let pascal = Ident::new(&pascal, Span::call_site());
        tokens.extend(quote! { #pascal })
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ItemName(String);

impl From<String> for ItemName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for ItemName {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl ToTokens for ItemName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let snake = self.0.to_snake_case();

        let ident = match syn::parse_str::<Ident>(&snake) {
            Ok(ident) => ident,
            Err(_) => {
                let input = format!("r#{}", snake);
                syn::parse_str::<Ident>(&input).unwrap()
            }
        };

        tokens.extend(quote! { #ident })
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Deprecated {
    #[serde(rename = "isDeprecated")]
    pub is_deprecated: Option<bool>,

    #[serde(rename = "deprecationReason")]
    pub deprecation_reason: Option<String>,
}

impl ToTokens for Deprecated {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self.is_deprecated {
            None | Some(false) => quote! {},
            _ => match self.deprecation_reason.as_ref() {
                Some(reason) => quote! { #[deprecated=#reason] },
                None => quote! { #[deprecated] },
            },
        })
    }
}
