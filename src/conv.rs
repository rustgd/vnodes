//! Conversion traits for type -> value and value -> type

use raw::RawValue;
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

impl<'a, A, B> ValueConv<'a> for (A, B)
where
    A: ValueConv<'a>,
    B: ValueConv<'a>,
{
    fn from_value(value: Value<'a>) -> Result<Self> {
        fn conv<'a, T>(raw: Option<&RawValue>) -> Result<T>
        where
            T: ValueConv<'a>,
        {
            raw.ok_or(Error::InvalidArgumentTypes)
                .and_then(|raw| unsafe { Value::from_raw(*raw).into_res() })
                .and_then(|val| T::from_value(val))
        }

        match value {
            Value::ValueArrayRef(raw) => Ok((conv(raw.get(0))?, conv(raw.get(1))?)),
            Value::ValueArray(ref raw) => Ok((conv(raw.get(0))?, conv(raw.get(1))?)),
            _ => Err(Error::WrongType),
        }
    }

    fn into_value(self) -> Value<'a> {
        // TODO: allow without boxing?

        let a: RawValue = self.0.into_value().into();
        let b: RawValue = self.1.into_value().into();
        let v: Vec<RawValue> = vec![a, b];

        Value::ValueArray(v.into_boxed_slice())
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
