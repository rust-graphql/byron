#[cfg(test)]
mod base;

#[cfg(test)]
mod tests;

pub mod shared;
pub mod types;

pub use proc_macro2::TokenStream;
pub use quote::ToTokens;

use proc_macro2::*;
use quote::*;
use serde::Deserialize;
use string_morph::Morph;

#[derive(Deserialize, Debug)]
pub struct Document {
    pub data: Data,
}

impl ToTokens for Document {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote! {
            #![allow(unused_variables)]
            use super::*;
        });

        self.data.to_tokens(tokens)
    }
}

#[derive(Deserialize, Debug)]
pub struct Data {
    #[serde(rename = "__schema")]
    pub schema: Schema,
}

impl ToTokens for Data {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.schema.to_tokens(tokens)
    }
}

#[derive(Deserialize, Debug)]
pub struct Schema {
    #[serde(rename = "queryType")]
    pub query_type: Option<QueryType>,

    #[serde(rename = "mutationType")]
    pub mutation_type: Option<MutationType>,

    #[serde(rename = "subscriptionType")]
    pub subscription_type: Option<SubscriptionType>,

    #[serde(default)]
    pub types: Vec<types::full::Type>,

    #[serde(default)]
    pub directives: Vec<Directive>,
}

impl ToTokens for Schema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for fulltype in &self.types {
            fulltype.to_tokens(tokens);
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct QueryType {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct MutationType {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct SubscriptionType {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Directive {
    pub name: shared::ItemName,

    pub description: shared::Documentation,

    #[serde(default)]
    pub locations: Vec<Location>,

    #[serde(default)]
    pub args: Vec<Input>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum Location {
    #[serde(rename = "FIELD")]
    Field,

    #[serde(rename = "FRAGMENT_SPREAD")]
    FragmentSpread,

    #[serde(rename = "INLINE_FRAGMENT")]
    InlineFragment,

    #[serde(rename = "FIELD_DEFINITION")]
    FieldDefinition,

    #[serde(rename = "ENUM_VALUE")]
    EnumValue,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Field {
    pub name: shared::ItemName,

    pub description: shared::Documentation,

    #[serde(flatten)]
    pub deprecated: shared::Deprecated,

    #[serde(default)]
    pub args: Vec<Input>,

    #[serde(rename = "type")]
    pub kind: types::nullable::Type,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Input {
    pub name: shared::ItemName,

    pub description: shared::Documentation,

    #[serde(rename = "type")]
    pub kind: types::nullable::Type,

    #[serde(rename = "defaultValue")]
    pub default_value: Option<String>,
}

impl ToTokens for Input {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut typ = TokenStream::default();
        self.kind.to_tokens(&mut typ);
        let name = &self.name;

        tokens.extend(quote! { #name: #typ })
    }
}
