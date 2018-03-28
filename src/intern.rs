#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Interned(pub u64);

impl Interned {
    pub fn into_inner(self) -> u64 {
        self.0
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

pub type InternedPath = [u64];

#[repr(C)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InternedPathBuf(Vec<u64>);

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
                .map(Interned::into_inner)
                .collect(),
        )
    }

    pub fn from_str(s: &str) -> Self {
        Self::from(s)
    }

    pub fn path(&self) -> &InternedPath {
        &self.0
    }
}
