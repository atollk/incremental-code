use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(FieldsAs, attributes(skip_field, fields_as))]
pub fn derive_fields_as(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Find #[fields_as(SomeTrait)] and parse the trait path
    let trait_path: syn::Path = input
        .attrs
        .iter()
        .find(|a| a.path().is_ident("fields_as"))
        .expect("missing #[fields_as(Trait)]")
        .parse_args()
        .expect("expected #[fields_as(TraitPath)]");

    let fields = match input.data {
        Data::Struct(s) => match s.fields {
            Fields::Named(f) => f.named,
            _ => panic!("FieldsAs requires named fields"),
        },
        _ => panic!("FieldsAs requires a struct"),
    };

    let idents: Vec<_> = fields
        .iter()
        .filter(|f| !f.attrs.iter().any(|a| a.path().is_ident("skip_field")))
        .map(|f| f.ident.clone().unwrap())
        .collect();
    let n = idents.len();

    quote! {
        impl #name {
            pub fn fields_as(&self) -> [&dyn #trait_path; #n] {
                [ #( &self.#idents as &dyn #trait_path ),* ]
            }

            pub fn fields_as_mut(&mut self) -> [&mut dyn #trait_path; #n] {
                [ #( &mut self.#idents as &mut dyn #trait_path ),* ]
            }
        }
    }
    .into()
}
