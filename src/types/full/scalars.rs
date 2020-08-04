use super::*;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Type {
    pub name: shared::TypeName,

    pub description: shared::Documentation,
}

impl ToTokens for Type {
    fn to_tokens(&self, _: &mut TokenStream) {}
}
