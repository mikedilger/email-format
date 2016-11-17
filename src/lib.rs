
extern crate buf_read_ext;

#[cfg(test)]
mod tests;

pub mod rfc5322;

/// Attempt to construct `Self` via a conversion.
///
/// This TryFrom trait is defined in the rust std library but is behind a
/// feature gate.  We place it here so that people using stable compilers
/// can still use our crate.  In the future, the std trait should be used.
pub trait TryFrom<T>: Sized {
    /// The type returned in the event of a conversion error.
    type Err;

    /// Performs the conversion.
    fn try_from(T) -> Result<Self, Self::Err>;
}

// We implement TryFrom from T to T with our ParseError for crate ergonomics
// (Rust won't let it be implemented with an unconstrained error type)
impl<T> TryFrom<T> for T {
    type Err = ::rfc5322::error::ParseError;
    fn try_from(input: T) -> Result<T, Self::Err> {
        Ok(input)
    }
}
