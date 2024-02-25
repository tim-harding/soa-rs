use soapy::Soapy;

#[derive(Debug, Clone, Copy, PartialEq, Soapy)]
#[extra_impl(Debug, PartialEq)]
struct Alignment {
    #[align(64)]
    a: f32,
    #[align(64)]
    b: f32,
    #[align(64)]
    c: f32,
    #[align(64)]
    d: f32,
}

fn main() {}
