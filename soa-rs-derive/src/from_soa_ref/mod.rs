use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, Generics, Ident, parse_macro_input, spanned::Spanned};

pub fn from_soa_ref(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    match from_soa_ref_derive(input) {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error(),
    }
    .into()
}

fn from_soa_ref_derive(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = input;

    let strukt = match data {
        Data::Struct(strukt) => strukt,
        Data::Enum(_) | Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                ident,
                "FromSoaRef only applies to structs",
            ));
        }
    };

    let fields = strukt.fields;

    // Get field identifiers
    let field_idents: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            field
                .ident
                .clone()
                .map(syn::Member::Named)
                .unwrap_or_else(|| {
                    syn::Member::Unnamed(syn::Index {
                        index: i as u32,
                        span: field.span(),
                    })
                })
        })
        .collect();

    generate_impl(ident, generics, field_idents)
}

fn generate_impl(
    ident: Ident,
    generics: Generics,
    field_idents: Vec<syn::Member>,
) -> Result<TokenStream2, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::soa_rs::FromSoaRef for #ident #ty_generics #where_clause {
            fn from_soa_ref(item: <Self as ::soa_rs::Soars>::Ref<'_>) -> Self {
                Self {
                    #(
                    #field_idents: item.#field_idents.clone(),
                    )*
                }
            }
        }
    })
}
