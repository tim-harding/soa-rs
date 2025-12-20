use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input, spanned::Spanned};

pub fn from_soa_ref(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    match input_to_tokens(input) {
        Ok(tokens) => tokens,
        Err(e) => e.into_compile_error(),
    }
    .into()
}

fn input_to_tokens(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let DeriveInput {
        ident,
        data,
        generics,
        ..
    } = input;

    let fields = match data {
        Data::Struct(strukt) => strukt.fields,
        Data::Enum(_) | Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                ident,
                "FromSoaRef only applies to structs",
            ));
        }
    };

    let members: Vec<_> = fields
        .into_iter()
        .enumerate()
        .map(|(i, field)| match field.ident {
            Some(ident) => syn::Member::Named(ident),
            None => syn::Member::Unnamed(syn::Index {
                index: i as u32,
                span: field.span(),
            }),
        })
        .collect();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::soa_rs::FromSoaRef for #ident #ty_generics #where_clause {
            fn from_soa_ref(item: <Self as ::soa_rs::Soars>::Ref<'_>) -> Self {
                Self {
                    #(
                    #members: item.#members.clone(),
                    )*
                }
            }
        }
    })
}
