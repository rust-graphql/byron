use super::*;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Type {
    pub name: shared::TypeName,

    pub description: shared::Documentation,

    #[serde(rename = "inputFields", default)]
    pub fields: Vec<Input>,
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let docs = &self.description;
        let name = &self.name;

        let fields = self.fields.iter().map(|field| {
            let docs = &field.description;
            let orig = &*field.name;
            let name = &field.name;
            let kind = &field.kind;

            quote! {
                #[serde(rename = #orig)]
                #docs
                pub #name: #kind
            }
        });

        tokens.extend(quote! {
            #docs
            #[derive(Debug, Deserialize, Serialize)]
            pub struct #name {
                #(#fields),*
            }
        })
    }
}
