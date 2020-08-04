use super::*;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Type {
    pub name: shared::TypeName,

    pub description: shared::Documentation,

    #[serde(default)]
    pub fields: Vec<Field>,

    pub interfaces: Option<Vec<crate::types::interface::parent::Type>>,
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let docs = &self.description;
        let name = &self.name;

        fn embedded(field: &Field) -> bool {
            use super::super::list::Type as ListType;
            use super::super::nullable::Type as NullableType;

            if !field.args.is_empty() {
                return false;
            }

            match &field.kind {
                NullableType::Scalar { .. } => true,
                NullableType::Enum { .. } => true,
                NullableType::NonNull { of } => match &**of {
                    ListType::Scalar { .. } => true,
                    ListType::Enum { .. } => true,
                    _ => false,
                },
                _ => false,
            }
        }

        let fields = self.fields.iter().filter(|x| embedded(x)).map(|field| {
            let deprecated = &field.deprecated;
            let docs = &field.description;
            let orig = &*field.name;
            let name = &field.name;
            let kind = &field.kind;

            quote! {
                #[serde(rename = #orig)]
                #deprecated
                #docs
                pub #name: #kind
            }
        });

        let methods = self.fields.iter().filter(|x| !embedded(x)).map(|field| {
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
            #[derive(Debug, Deserialize, Serialize)]
            pub struct #name {
                #(#fields),*
            }

            impl #name {
                #(#methods { unimplemented!() })*
            }
        })
    }
}
