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

    match data {
        Data::Struct(strukt) => {
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

            generate_impl(ident, generics, fields, field_idents)
        }
        Data::Enum(_) | Data::Union(_) => Err(syn::Error::new_spanned(
            ident,
            "FromSoaRef only applies to structs",
        )),
    }
}

fn generate_impl(
    ident: Ident,
    generics: Generics,
    fields: syn::Fields,
    field_idents: Vec<syn::Member>,
) -> Result<TokenStream2, syn::Error> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate the struct construction based on field type
    let construction = match fields {
        syn::Fields::Named(_) => {
            // For named structs, use field: value syntax
            let field_clones = field_idents.iter().map(|ident| {
                quote! {
                    #ident: item.#ident.clone()
                }
            });
            quote! {
                Self {
                    #(#field_clones),*
                }
            }
        }
        syn::Fields::Unnamed(_) => {
            // For tuple structs, just use values without field names
            let field_clones = field_idents.iter().map(|ident| {
                quote! {
                    item.#ident.clone()
                }
            });
            quote! {
                Self(#(#field_clones),*)
            }
        }
        syn::Fields::Unit => {
            quote! { Self }
        }
    };

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics ::soa_rs::FromSoaRef for #ident #ty_generics #where_clause {
            fn from_soa_ref(item: <Self as ::soa_rs::Soars>::Ref<'_>) -> Self {
                #construction
            }
        }
    })
}
