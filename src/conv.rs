//! Conversion traits for type -> value and value -> type

use {Error, Interned, NodeHandle, NodeHandleRef, Result, Value};

pub trait ValueConv<'a>: Sized {
    fn from_value(value: Value<'a>) -> Result<Self>;
    fn into_value(self) -> Value<'a>;
}

impl<'a> ValueConv<'a> for () {
    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Void => Ok(()),
            _ => Err(Error::WrongType),
        }
    }

    fn into_value(self) -> Value<'a> {
        Value::Void
    }
}

impl<'a> ValueConv<'a> for Value<'a> {
    fn from_value(value: Value<'a>) -> Result<Self> {
        // Check if it's a value
        Ok(value)
    }

    fn into_value(self) -> Value<'a> {
        self
    }
}

macro_rules! impl_value_conv {
    ($ty:ident, $variant:ident $(($lt:tt))*) => {
        impl<'a> ValueConv<'a> for ($ty$(<$lt>)*) {
            fn from_value(value: Value<'a>) -> Result<Self> {
                match value {
                    Value::$variant(value) => Ok(value),
                    _ => Err(Error::WrongType),
                }
            }

            fn into_value(self) -> Value<'a> {
                Value::$variant(self)
            }
        }
    };
}

impl_value_conv!(i64, Signed);
impl_value_conv!(u64, Unsigned);
impl_value_conv!(bool, Bool);
impl_value_conv!(f64, Float);
impl_value_conv!(Interned, Interned);
impl_value_conv!(NodeHandle, Node);
impl_value_conv!(NodeHandleRef, NodeRef ('a));
