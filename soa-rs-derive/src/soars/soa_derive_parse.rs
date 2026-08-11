use crate::soars::soa_derive::{SoaDerive, SoaDeriveMask};

#[derive(Debug, Clone, Default)]
pub struct SoaDeriveParse {
    r#ref: Vec<syn::Path>,
    ref_mut: Vec<syn::Path>,
    ref_packed: Option<Vec<syn::Path>>,
    ref_mut_packed: Option<Vec<syn::Path>>,
    slices: Vec<syn::Path>,
    slices_mut: Vec<syn::Path>,
    array: Vec<syn::Path>,
}

impl SoaDeriveParse {
    pub fn into_derive(self) -> SoaDerive {
        let Self {
            r#ref: reff,
            ref_mut,
            ref_packed,
            ref_mut_packed,
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
            ref_packed: ref_packed.map(|ref_packed| {
                quote::quote! {
                    #[derive(#(#ref_packed),*)]
                }
            }),
            ref_mut_packed: ref_mut_packed.map(|ref_mut_packed| {
                quote::quote! {
                    #[derive(#(#ref_mut_packed),*)]
                }
            }),
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
        // Carry packed enabled state across multiple `#[soa_derive(...)]` occurrences.
        mask.ref_packed = self.ref_packed.is_some();
        mask.ref_mut_packed = self.ref_mut_packed.is_some();
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("include") {
                mask = SoaDeriveMask::splat(false);
                meta.parse_nested_meta(|meta| {
                    mask.set_by_path(&meta.path, true).map_err(|_| {
                        meta.error(format!("unknown include specifier {:?}", meta.path))
                    })
                })?;
                if mask.ref_packed {
                    self.ref_packed.get_or_insert_with(Vec::new);
                }
                if mask.ref_mut_packed {
                    self.ref_mut_packed.get_or_insert_with(Vec::new);
                }
            } else if meta.path.is_ident("exclude") {
                meta.parse_nested_meta(|meta| {
                    mask.set_by_path(&meta.path, false).map_err(|_| {
                        meta.error(format!("unknown exclude specifier {:?}", meta.path))
                    })
                })?;
                if !mask.ref_packed {
                    self.ref_packed = None;
                }
                if !mask.ref_mut_packed {
                    self.ref_mut_packed = None;
                }
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
            .chain(
                mask.ref_packed
                    .then_some(self.ref_packed.as_mut())
                    .flatten(),
            )
            .chain(
                mask.ref_mut_packed
                    .then_some(self.ref_mut_packed.as_mut())
                    .flatten(),
            )
            .chain(mask.slice.then_some(&mut self.slices))
            .chain(mask.slice_mut.then_some(&mut self.slices_mut))
            .chain(mask.array.then_some(&mut self.array));

        for set in to_extend {
            set.extend(collected.iter().cloned());
        }

        Ok(())
    }
}
