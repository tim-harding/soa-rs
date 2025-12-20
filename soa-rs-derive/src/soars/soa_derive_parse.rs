use crate::soars::soa_derive::{SoaDerive, SoaDeriveMask};

#[derive(Debug, Clone, Default)]
pub struct SoaDeriveParse {
    r#ref: Vec<syn::Path>,
    ref_mut: Vec<syn::Path>,
    slices: Vec<syn::Path>,
    slices_mut: Vec<syn::Path>,
    array: Vec<syn::Path>,
}

impl SoaDeriveParse {
    pub fn into_derive(self) -> SoaDerive {
        let Self {
            r#ref: reff,
            ref_mut,
            slices,
            slices_mut,
            array,
        } = self;
        SoaDerive {
            r#ref: quote::quote! {
                #[derive(#(#reff),*)]
            },
            ref_mut: quote::quote! {
                #[derive(#(#ref_mut),*)]
            },
            slices: quote::quote! {
                #[derive(#(#slices),*)]
            },
            slices_mut: quote::quote! {
                #[derive(#(#slices_mut),*)]
            },
            array: quote::quote! {
                #[derive(#(#array),*)]
            },
        }
    }

    pub fn append(&mut self, attr: &syn::Attribute) -> Result<(), syn::Error> {
        let mut collected = vec![];
        let mut mask = SoaDeriveMask::new();
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("include") {
                mask = SoaDeriveMask::splat(false);
                meta.parse_nested_meta(|meta| {
                    mask.set_by_path(&meta.path, true).map_err(|_| {
                        meta.error(format!("unknown include specifier {:?}", meta.path))
                    })
                })?;
            } else if meta.path.is_ident("exclude") {
                meta.parse_nested_meta(|meta| {
                    mask.set_by_path(&meta.path, false).map_err(|_| {
                        meta.error(format!("unknown exclude specifier {:?}", meta.path))
                    })
                })?;
            } else {
                collected.push(meta.path);
            }
            Ok(())
        })?;

        let to_extend = mask
            .r#ref
            .then_some(&mut self.r#ref)
            .into_iter()
            .chain(mask.ref_mut.then_some(&mut self.ref_mut))
            .chain(mask.slice.then_some(&mut self.slices))
            .chain(mask.slice_mut.then_some(&mut self.slices_mut))
            .chain(mask.array.then_some(&mut self.array));

        for set in to_extend {
            set.extend(collected.iter().cloned());
        }

        Ok(())
    }
}
