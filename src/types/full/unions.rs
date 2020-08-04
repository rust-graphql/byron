use super::*;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Type {
    pub name: shared::TypeName,

    pub description: shared::Documentation,

    #[serde(rename = "possibleTypes", default)]
    pub possible: Vec<crate::types::named::Type>,
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let docs = &self.description;
        let name = &self.name;

        let vars = self.possible.iter().map(|x| {
            let orig = x.orig();

            quote! {
                #[serde(rename = #orig)]
                #x(#x)
            }
        });

        tokens.extend(quote! {
            #docs
            #[derive(Debug, Deserialize, Serialize)]
            pub enum #name {
                #(#vars),*
            }
        });
    }
}
