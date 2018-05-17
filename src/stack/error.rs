use stack::Ty;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Clone, Debug, Fail, PartialEq)]
pub enum Error {
    #[fail(display = "Other")]
    External,
    #[fail(display = "Stack too small: {}", _0)]
    StackTooSmall(StackTooSmall),
    #[fail(display = "Tuple mismatch: {}", _0)]
    TupleMismatch(TupleMismatch),
    #[fail(display = "Unexpected type: {}", _0)]
    UnexpectedTy(UnexpectedTy),
}

impl From<StackTooSmall> for Error {
    fn from(e: StackTooSmall) -> Self {
        Error::StackTooSmall(e)
    }
}

impl From<TupleMismatch> for Error {
    fn from(e: TupleMismatch) -> Self {
        Error::TupleMismatch(e)
    }
}

impl From<UnexpectedTy> for Error {
    fn from(e: UnexpectedTy) -> Self {
        Error::UnexpectedTy(e)
    }
}

#[derive(Clone, Debug, Fail, PartialEq)]
#[fail(display = "Stack too small for reading a {}", what)]
pub struct StackTooSmall {
    pub what: &'static str,
}

#[derive(Clone, Debug, Fail, PartialEq)]
#[fail(display = "Unexpected {} elements, got {}", expected, got)]
pub struct TupleMismatch {
    pub expected: usize,
    pub got: usize,
}

#[derive(Clone, Debug, Fail, PartialEq)]
#[fail(display = "Unexpected type: expected {}, got {}", expected, got)]
pub struct UnexpectedTy {
    pub expected: Ty,
    pub got: Ty,
}
