use crate::soars::{soa_derive::SoaDerive, soa_derive_parse::SoaDeriveParse};

#[derive(Debug, Clone)]
pub struct SoaAttrs {
    pub derive: SoaDerive,
    pub include_array: bool,
}

impl SoaAttrs {
    pub fn new(attributes: &[syn::Attribute]) -> Result<Self, syn::Error> {
        let mut derive_parse = SoaDeriveParse::default();
        let mut include_array = false;
        for attr in attributes {
            let path = attr.path();
            if path.is_ident("soa_derive") {
                derive_parse.append(attr)?;
            } else if path.is_ident("soa_array") {
                include_array = true;
            }
        }

        Ok(Self {
            derive: derive_parse.into_derive(),
            include_array,
        })
    }
}
