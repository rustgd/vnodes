use std::str::from_utf8_unchecked;

#[derive(Debug)]
pub enum Ident<'a> {
    Interned(Interned),
    String(String),
    StringRef(&'a str),
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Interned(pub u64);

impl Interned {
    pub fn into_inner(self) -> u64 {
        self.0
    }

    pub fn un_intern<'a>(&self, buf: &'a mut [u8]) -> &'a str {
        let num = self.un_intern_raw(buf);

        unsafe { from_utf8_unchecked(&buf[..num]) }
    }

    pub fn un_intern_raw(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= 10);

        let array = [
            b'\0', b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm',
            b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'0',
            b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'_', b'.',
        ];

        let shifts = [0x36, 0x30, 0x2A, 0x24, 0x1E, 0x18, 0x12, 0xC, 0x6, 0x0];

        let masks = [
            0x3F << shifts[0],
            0x3F << shifts[1],
            0x3F << shifts[2],
            0x3F << shifts[3],
            0x3F << shifts[4],
            0x3F << shifts[5],
            0x3F << shifts[6],
            0x3F << shifts[7],
            0x3F << shifts[8],
            0x3F << shifts[9],
        ];

        let tmp = self.0;
        let mut ind = 0;
        let mut buf_ind = 0;
        loop {
            if ind == 10 {
                break buf_ind;
            }

            let code = (tmp & masks[ind]) >> shifts[ind];
            ind += 1;

            if code == 0 {
                continue;
            }

            buf[buf_ind] = array[code as usize];
            buf_ind += 1;
        }
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
        b'a'...b'z' => 1 + b - b'a',
        b'A'...b'Z' => 1 + b - b'A',
        b'0'...b'9' => b - b'0' + 26 + 1,
        b'-' | b'_' => 37,
        b'.' => 38,
        _ => panic!("Unsupported: {}", b as char),
    }
}

fn intern(mut s: &[u8]) -> u64 {
    let mut result = 0;

    while let Some(&byte) = s.get(0) {
        result <<= 6;
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

impl InternedPathBuf {
    pub fn from_iter<I, T>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Interned>,
    {
        InternedPathBuf(iter.into_iter().map(Into::into).collect())
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

    pub fn pop(&mut self) -> Option<Interned> {
        self.0.pop()
    }
}

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

    fn check_same(s: &str) {
        let interned = Interned::from(s);
        let mut un_interned = [0; 10];
        let un_interned = interned.un_intern(&mut un_interned);
        println!("{}", un_interned);
        assert_eq!(interned, Interned::from(un_interned));
    }

    fn check_exact_same(s: &str) {
        let interned = Interned::from(s);
        let mut un_interned = [0; 10];
        let un_interned = interned.un_intern(&mut un_interned);
        assert_eq!(s, un_interned);
    }

    #[test]
    fn check_idents() {
        check_same("my-world");
        check_same("WhAtEvEr");
        check_same("conf.ron");

        check_exact_same("exact_str");
        check_exact_same("my.world");
        check_exact_same("007");
    }
}
