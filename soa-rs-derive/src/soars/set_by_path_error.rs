use core::{
    error::Error,
    fmt::{self, Display, Formatter},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SetByPathError;

impl Display for SetByPathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "unknown mask specifier")
    }
}

impl Error for SetByPathError {}
