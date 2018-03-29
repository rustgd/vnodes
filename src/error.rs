#[derive(Clone, Copy, Debug, Fail, PartialEq)]
#[fail(display = "Vnodes error")]
#[repr(u8)]
pub enum Error {
    #[fail(display = "Action not supported")]
    ActionNotSupported = 0x1,
    #[fail(display = "Expected node")]
    ExpectedNode = 0x2,
    #[fail(display = "Invalid argument types")]
    InvalidArgumentTypes = 0x3,
    #[fail(display = "No such entry")]
    NoSuchEntry = 0x4,
    #[fail(display = "No such entry")]
    PathEmpty = 0x5,
    #[fail(display = "Unknown type")]
    UnknownTypeFlags = 0x6,
    #[fail(display = "Wrong type")]
    WrongType = 0x7, // TODO: `ExpectedNode` and `WrongType` intersect
}

pub type Result<T> = ::std::result::Result<T, Error>;
