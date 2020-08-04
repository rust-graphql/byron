use super::*;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Type {
    #[serde(flatten)]
    pub object: super::objects::Type,

    #[serde(rename = "possibleTypes")]
    pub possible: Vec<crate::types::interface::child::Type>,
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let docs = &self.object.description;
        let name = &self.object.name;

        let traits = match self.object.interfaces.as_ref() {
            Some(traits) if !traits.is_empty() => quote! {: #(#traits),* },
            _ => quote! {},
        };

        let fields = self.object.fields.iter().map(|field| {
            let deprecated = &field.deprecated;
            let docs = &field.description;
            let name = &field.name;

            let mut typ = TokenStream::default();
            field.kind.to_tokens(&mut typ);

            let args = &field.args;

            quote! {
                #docs
                #deprecated
                pub fn #name(&self #(, #args)*) -> #typ
            }
        });

        tokens.extend(quote! {
            #docs
            pub trait #name #traits {
                #(#fields;)*
            }
        })
    }
}
