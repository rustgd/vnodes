#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Interned(pub u64);

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
