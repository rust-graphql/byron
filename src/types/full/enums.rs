use super::*;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Value {
    pub name: shared::TypeName,

    pub description: shared::Documentation,

    #[serde(flatten)]
    deprecated: crate::shared::Deprecated,
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let docs = &self.description;
        let depr = &self.deprecated;
        let orig = &*self.name;
        let name = &self.name;

        tokens.extend(quote! {
            #[serde(rename = #orig)]
            #depr
            #docs
            #name
        })
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Type {
    pub name: shared::TypeName,

    pub description: shared::Documentation,

    #[serde(rename = "enumValues", default)]
    pub values: Vec<Value>,
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let docs = &self.description;
        let vals = &self.values;
        let name = &self.name;

        tokens.extend(quote! {
            #docs
            #[derive(Debug, Deserialize, Serialize)]
            pub enum #name {
                #(#vals,)*
            }
        });
    }
}
