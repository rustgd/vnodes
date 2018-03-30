//! Conversion traits for type -> value and value -> type

use raw::RawValue;
use {Error, Interned, InternedPath, InternedPathBuf, NodeHandle, NodeHandleRef, Result, Value};

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

macro_rules! impl_conv_tuple {
    ($($tys:ident . $field:tt)*) => {
        impl<'a, $($tys,)*> ValueConv<'a> for ($($tys,)*)
        where
            $($tys : ValueConv<'a>,)*
        {
            fn from_value(value: Value<'a>) -> Result<Self> {
                match value {
                    Value::ValueArrayRef(raw) =>
                        Ok(($(conv_opt(raw.get($field))?,)*)),
                    Value::ValueArray(ref raw) =>
                        Ok(($(conv_opt(raw.get($field))?,)*)),
                    _ => Err(Error::WrongType),
                }
            }

            fn into_value(self) -> Value<'a> {
                // TODO: allow without boxing?

                $(
                    #[allow(non_snake_case)]
                    let $tys: RawValue = self.$field.into_value().into();
                )*
                let v: Vec<RawValue> = vec![$($tys),*];

                Value::ValueArray(v.into_boxed_slice())
            }
        }
    };
}

impl_conv_tuple!(A.0 B.1);
impl_conv_tuple!(A.0 B.1 C.2);
impl_conv_tuple!(A.0 B.1 C.2 D.3);
impl_conv_tuple!(A.0 B.1 C.2 D.3 E.4);

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

// Typedef to make macro call below work
type InternedPathRef<'a> = &'a InternedPath;

impl_value_conv!(i64, Signed);
impl_value_conv!(u64, Unsigned);
impl_value_conv!(bool, Bool);
impl_value_conv!(f64, Float);
impl_value_conv!(Interned, Interned);
impl_value_conv!(InternedPathBuf, InternedPathBuf);
impl_value_conv!(InternedPathRef, InternedPathRef ('a));
impl_value_conv!(NodeHandle, Node);
impl_value_conv!(NodeHandleRef, NodeRef ('a));

fn conv_opt<'a, T>(raw: Option<&RawValue>) -> Result<T>
where
    T: ValueConv<'a>,
{
    raw.ok_or(Error::InvalidArgumentTypes)
        .and_then(|raw| unsafe { Value::from_raw(*raw).into_res() })
        .and_then(|val| T::from_value(val))
}

#[cfg(test)]
mod tests {
    use super::*;
    use MapNode;
    use std::fmt::Debug;

    /// Convert `T` -> `Value` -> `RawValue` -> `Value` -> `T` and check for equality.
    fn check_equal<'a, T>(start: T)
    where
        T: Clone + Debug + PartialEq + ValueConv<'a> + 'a,
    {
        let value = start.clone().into_value();
        let raw: RawValue = value.into();
        let value = unsafe { Value::from_raw(raw) };
        let end = ValueConv::from_value(value);

        assert_eq!(end, Ok(start));
    }

    #[test]
    fn check_primitives() {
        check_equal(99u64);
        check_equal(43i64);
        check_equal(-43i64);
        check_equal(-3.14f64);
        check_equal(true);
        check_equal(false);
        check_equal(Interned::from("my_ident"));
    }

    #[test]
    fn check_tuples() {
        check_equal((5u64, 19i64));
        check_equal((5u64, true, -91.0f64));
        check_equal((
            5u64,
            (Interned::from("nested"), Interned::from("tuples")),
            -91.0f64,
        ));
    }

    #[test]
    fn check_paths() {
        check_equal(Interned::from("simple"));
        check_equal(InternedPathBuf::from("simple/path/to/hell"));
    }

    #[test]
    fn check_nodes() {
        check_equal(MapNode::new_node());
        let node = MapNode::new_node();
        check_equal(node.handle_ref());
    }
}
