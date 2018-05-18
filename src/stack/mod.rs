pub use self::error::{Error, Result, StackTooSmall, TupleMismatch, UnexpectedTy};
pub use self::value::{Pop, Push};

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::mem::{transmute, uninitialized};

mod error;
mod value;

pub struct Stack {
    inner: Vec<u8>,
}

impl Stack {
    pub fn new() -> Self {
        Stack { inner: vec![] }
    }

    pub fn push<V>(&mut self, value: V)
    where
        V: Push,
    {
        V::push(self, value);
    }

    fn push_tag(&mut self, tag: Ty) {
        self.push_untagged_bytes(&[tag as u8]);
    }

    fn push_u64(&mut self, v: u64) {
        unsafe {
            let bytes: [u8; 8] = transmute(v);

            self.push_untagged_bytes(&bytes);
        }
    }

    fn push_untagged_bytes(&mut self, bytes: &[u8]) {
        self.inner.extend(bytes);
    }

    pub fn pop<V>(&mut self) -> Result<V>
    where
        V: Pop,
    {
        V::pop(self)
    }

    fn peek_tag(&mut self) -> Result<Ty> {
        let ty = Ty::try_from(*self.inner.last().ok_or(StackTooSmall { what: "tag" })?)
            .expect("Expected a tag at this position, stack might be malformed");

        Ok(ty)
    }

    fn pop_u64(&mut self) -> u64 {
        unsafe {
            let mut bytes: [u8; 8] = uninitialized();

            self.pop_untagged_bytes(bytes.as_mut_ptr(), 8);

            transmute(bytes)
        }
    }

    fn pop_untagged_byte(&mut self) -> u8 {
        self.inner.pop().unwrap()
    }

    unsafe fn pop_untagged_bytes(&mut self, dest: *mut u8, len: usize) {
        use std::ptr::copy_nonoverlapping;

        assert!(self.inner.len() >= len);

        let start = self.inner.len() - len;
        copy_nonoverlapping(self.inner.as_ptr().offset(start as isize), dest, len);
        self.inner.set_len(start);
    }

    fn expect_tag(&mut self, tag: Ty) -> Result<()> {
        let got = self.peek_tag()?;
        if got == tag {
            self.pop_untagged_byte();
            Ok(())
        } else {
            Err(UnexpectedTy { expected: tag, got }.into())
        }
    }
}

impl Drop for Stack {
    fn drop(&mut self) {
        // TODO: drop remaining elements
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Ty {
    /// reverse: tag, element type, len, (element)*
    Array = 0x1,
    Bool = 0x2,
    Float = 0x3,
    Int = 0x4,
    Interned = 0x5,
    String = 0x6,
    /// reverse: tuple tag, len (> 0), (types)*, (element)*
    Tuple = 0x7,
    Uint = 0x8,
    Void = 0x9,
}

impl Display for Ty {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

impl Ty {
    pub fn try_from(byte: u8) -> Option<Self> {
        match byte {
            0x1 => Some(Ty::Array),
            0x2 => Some(Ty::Bool),
            0x3 => Some(Ty::Float),
            0x4 => Some(Ty::Int),
            0x5 => Some(Ty::Interned),
            0x6 => Some(Ty::String),
            0x7 => Some(Ty::Tuple),
            0x8 => Some(Ty::Uint),
            0x9 => Some(Ty::Void),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ty_from_byte() {
        for i in 0..=0x9 {
            assert_eq!(i, Ty::try_from(i).unwrap() as u8);
        }
    }
}
