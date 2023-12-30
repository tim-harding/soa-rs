pub use soapy_derive::Soapy;

#[cfg(test)]
mod tests {
    use crate::Soapy;

    #[derive(Soapy, Debug, Clone, Copy, PartialEq, Eq)]
    struct El {
        foo: u64,
        bar: u8,
        baz: [u32; 2],
    }

    const A: El = El {
        foo: 20,
        bar: 10,
        baz: [6, 4],
    };

    const B: El = El {
        foo: 10,
        bar: 5,
        baz: [3, 2],
    };

    const ZERO: El = El {
        foo: 0,
        bar: 0,
        baz: [0, 0],
    };
}
