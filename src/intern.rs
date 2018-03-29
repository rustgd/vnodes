#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Interned(pub u64);

impl Interned {
    pub fn into_inner(self) -> u64 {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl<'a> From<&'a Interned> for Interned {
    fn from(x: &'a Interned) -> Self {
        *x
    }
}

impl From<u64> for Interned {
    fn from(x: u64) -> Self {
        Interned(x)
    }
}

impl<'a> From<&'a u64> for Interned {
    fn from(x: &'a u64) -> Self {
        Interned(*x)
    }
}

impl<'a> From<&'a [u8]> for Interned {
    fn from(bytes: &[u8]) -> Self {
        Interned(intern(bytes))
    }
}

impl<'a> From<&'a str> for Interned {
    fn from(s: &str) -> Self {
        Interned::from(s.as_bytes())
    }
}

fn intern_byte(b: u8) -> u8 {
    match b {
        b'a'...b'z' => b - b'a',
        b'A'...b'Z' => b - b'A',
        b'0'...b'9' => b - b'0' + 26,
        b'-' | b'_' => 36,
        b'.' => 37,
        _ => panic!(),
    }
}

fn intern(mut s: &[u8]) -> u64 {
    let mut result = 0;

    while let Some(&byte) = s.get(0) {
        result <<= 5;
        result |= intern_byte(byte) as u64;
        s = &s[1..];
    }

    result
}

// TODO: decide what to do here
// TODO: From impls, construction
// TODO: `Borrow` implementation
// it would be nice to have a borrowed version, however
// * DSTs can't be handled very well
// * for this slice we cannot have an impl block
pub type InternedPath = [Interned];

#[repr(C)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InternedPathBuf(Vec<Interned>);

impl From<Box<[Interned]>> for InternedPathBuf {
    fn from(slice: Box<[Interned]>) -> Self {
        InternedPathBuf(slice.into_vec())
    }
}

impl<'a, T> From<&'a [T]> for InternedPathBuf
where
    &'a T: Into<Interned>,
{
    fn from(slice: &'a [T]) -> Self {
        InternedPathBuf::from_iter(slice)
    }
}

impl<'a> From<&'a str> for InternedPathBuf {
    fn from(s: &str) -> Self {
        InternedPathBuf::from_iter(s.split('/'))
    }
}

impl InternedPathBuf {
    pub fn from_iter<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Interned>,
    {
        InternedPathBuf(
            iter.into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    pub fn from_str(s: &str) -> Self {
        Self::from(s)
    }

    pub fn into_boxed_slice(self) -> Box<[Interned]> {
        self.0.into_boxed_slice()
    }

    pub fn is_absolute(&self) -> bool {
        self.0.get(0).map(Interned::is_empty).unwrap_or(false)
    }

    pub fn path(&self) -> &InternedPath {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn check_no_pad() {
        assert_eq!(size_of::<Interned>(), size_of::<u64>());
    }

    #[test]
    fn simple_path() {
        let path = InternedPathBuf::from("very/simple/path");
        assert_eq!(path.path()[0], Interned::from("very"));
        assert_eq!(path.path()[1], Interned::from("simple"));
        assert_eq!(path.path()[2], Interned::from("path"));
    }

    #[test]
    fn absolute() {
        let path = InternedPathBuf::from("/this/is/an/absolute/path");
        assert!(path.is_absolute());
    }
}
