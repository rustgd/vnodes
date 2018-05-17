use stack::{Result, Stack, TupleMismatch, Ty};

pub trait Push: Sized {
    fn push_tags(stack: &mut Stack);
    fn push_value(stack: &mut Stack, value: Self);

    fn push(stack: &mut Stack, value: Self) {
        Self::push_value(stack, value);
        Self::push_tags(stack);
    }
}

pub trait Pop: Sized {
    /// Should be implemented in most cases, may panic if you implement `pop_tags` and
    /// `restore_tags` yourself.
    fn tag() -> Ty;

    fn pop_tags(stack: &mut Stack) -> Result<()> {
        stack.expect_tag(Self::tag())
    }

    fn restore_tags(stack: &mut Stack) {
        stack.push_tag(Self::tag());
    }

    fn pop_value(stack: &mut Stack) -> Self;

    fn pop(stack: &mut Stack) -> Result<Self> {
        Self::pop_tags(stack)?;

        Ok(Self::pop_value(stack))
    }
}

impl Push for () {
    fn push_tags(stack: &mut Stack) {
        stack.push_tag(Ty::Void);
    }

    fn push_value(_: &mut Stack, _: Self) {}
}

impl Pop for () {
    fn tag() -> Ty {
        Ty::Void
    }

    fn pop_value(_: &mut Stack) -> Self {
        ()
    }
}

impl Push for bool {
    fn push_tags(stack: &mut Stack) {
        stack.push_tag(Ty::Bool);
    }

    fn push_value(stack: &mut Stack, value: Self) {
        stack.push_untagged_bytes(&[value as u8]);
    }
}

impl Pop for bool {
    fn tag() -> Ty {
        Ty::Bool
    }

    fn pop_value(stack: &mut Stack) -> Self {
        match stack.pop_untagged_byte() {
            0 => false,
            1 => true,
            _ => unreachable!(),
        }
    }
}

impl Push for u64 {
    fn push_tags(stack: &mut Stack) {
        stack.push_tag(Ty::Uint);
    }

    fn push_value(stack: &mut Stack, value: Self) {
        stack.push_u64(value);
    }
}

impl Pop for u64 {
    fn tag() -> Ty {
        Ty::Uint
    }

    fn pop_value(stack: &mut Stack) -> Self {
        stack.pop_u64()
    }
}

impl Push for i64 {
    fn push_tags(stack: &mut Stack) {
        stack.push_tag(Ty::Int);
    }

    fn push_value(stack: &mut Stack, value: Self) {
        stack.push_u64(value as u64);
    }
}

impl Pop for i64 {
    fn tag() -> Ty {
        Ty::Int
    }

    fn pop_value(stack: &mut Stack) -> Self {
        stack.pop_u64() as i64
    }
}

impl<'a> Push for &'a str {
    fn push_tags(stack: &mut Stack) {
        stack.push_tag(Ty::String);
    }

    fn push_value(stack: &mut Stack, value: Self) {
        stack.push_untagged_bytes(value.as_bytes());
        stack.push_u64(value.as_bytes().len() as u64);
    }
}

impl Pop for String {
    fn tag() -> Ty {
        Ty::String
    }

    fn pop_value(stack: &mut Stack) -> Self {
        let len = stack.pop_u64() as usize;
        let mut v = Vec::with_capacity(len);

        let s = unsafe {
            stack.pop_untagged_bytes(v.as_mut_ptr(), len);
            v.set_len(len);

            String::from_utf8_unchecked(v)
        };

        s
    }
}

impl<T> Push for Vec<T>
where
    T: Push,
{
    fn push_tags(stack: &mut Stack) {
        T::push_tags(stack);
        stack.push_tag(Ty::Array);
    }

    fn push_value(stack: &mut Stack, value: Self) {
        let len = value.len();

        for elem in value.into_iter().rev() {
            T::push_value(stack, elem);
        }

        stack.push_u64(len as u64);
    }
}

impl<T> Pop for Vec<T>
where
    T: Pop,
{
    fn tag() -> Ty {
        unreachable!()
    }

    fn pop_tags(stack: &mut Stack) -> Result<()> {
        stack.expect_tag(Ty::Array)?;

        match T::pop_tags(stack) {
            Ok(_) => Ok(()),
            Err(e) => {
                stack.push_tag(Ty::Array);

                Err(e)
            }
        }
    }

    fn restore_tags(stack: &mut Stack) {
        T::restore_tags(stack);
        stack.push_tag(Ty::Array);
    }

    fn pop_value(stack: &mut Stack) -> Self {
        let len = stack.pop_u64() as usize;
        let mut v = Vec::with_capacity(len);

        for _ in 0..len {
            v.push(T::pop_value(stack));
        }

        v
    }
}

macro_rules! impl_tuple_and_array {
    ($num:expr;$($field:ident),*;$($rev:ident),*) => {
        impl<$($field,)*> Push for ($($field,)*)
        where
            $($field: Push),*
        {
            fn push_tags(stack: &mut Stack) {
                $(
                $rev::push_tags(stack);
                )*
                stack.push_u64($num as u64);
                stack.push_tag(Ty::Tuple);
            }

            fn push_value(stack: &mut Stack, value: Self) {
                #[allow(bad_style)]
                let ($($field,)*) = value;
                $(
                $rev::push_value(stack, $rev);
                )*
            }
        }

        impl<T> Push for [T; $num]
        where
            T: Push,
        {
            fn push_tags(stack: &mut Stack) {
                T::push_tags(stack);
                stack.push_tag(Ty::Array);
            }

            fn push_value(stack: &mut Stack, mut value: Self) {
                use std::mem::{forget, replace, uninitialized};
                use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};

                let mut res = Ok(());

                for i in (0..$num).rev() {
                    let elem = unsafe { replace(&mut value[i], uninitialized()) };

                    // We may not panic inside this loop, the array contains uninitialized memory
                    // and would be dropped in case of a panic. Instead, we delay the panic and
                    // resume it once the loop finished and we called `forget(value)`.
                    res = res.and_then(|_| catch_unwind(AssertUnwindSafe(|| {
                        T::push_value(stack, elem);
                    })));
                }

                forget(value);

                if let Err(payload) = res {
                    resume_unwind(payload);
                }

                stack.push_u64($num);
            }
        }

        impl<$($field,)*> Pop for ($($field,)*)
        where
            $($field: Pop),*
        {
            fn tag() -> Ty {
                unreachable!()
            }

            fn pop_tags(stack: &mut Stack) -> Result<()> {
                stack.expect_tag(Ty::Tuple)?;
                let num_elems = stack.pop_u64();
                if num_elems != $num as u64 {
                    // we need to re-push length and tuple tag
                    stack.push_u64(num_elems);
                    stack.push_tag(Ty::Tuple);

                    return Err(TupleMismatch {
                        expected: $num as usize,
                        got: num_elems as usize,
                    }.into());
                }

                impl_tuple_and_array!(@pop_tags stack num_elems;$($field)*;);

                Ok(())
            }

            fn pop_value(stack: &mut Stack) -> Self {
                ($($field::pop_value(stack),)*)
            }
        }
    };
    (@pop_tags $stack:ident $num_elems:ident;; $($pushed:ident)*) => {
    };
    (@pop_tags $stack:ident $num_elems:ident; $field0:ident $($field:ident)*; $($pushed:ident)*) => {
        match $field0::pop_tags($stack) {
            Ok(_) => {}
            Err(e) => {
                // restore popped tags
                $(
                    $pushed::restore_tags($stack);
                )*
                $stack.push_u64($num_elems);
                $stack.push_tag(Ty::Tuple);

                return Err(e);
            }
        }
        impl_tuple_and_array!(@pop_tags $stack $num_elems; $($field)*; $field0 $($pushed)*);
    };
}

impl_tuple_and_array!(1;A;A);
impl_tuple_and_array!(2;A,B;B,A);
impl_tuple_and_array!(3;A,B,C;C,B,A);
impl_tuple_and_array!(4;A,B,C,D;D,C,B,A);
impl_tuple_and_array!(5;A,B,C,D,E;E,D,C,B,A);
impl_tuple_and_array!(6;A,B,C,D,E,F;F,E,D,C,B,A);
impl_tuple_and_array!(7;A,B,C,D,E,F,G;G,F,E,D,C,B,A);
impl_tuple_and_array!(8;A,B,C,D,E,F,G,H;H,G,F,E,D,C,B,A);
impl_tuple_and_array!(9;A,B,C,D,E,F,G,H,I;I,H,G,F,E,D,C,B,A);
impl_tuple_and_array!(10;A,B,C,D,E,F,G,H,I,J;J,I,H,G,F,E,D,C,B,A);
impl_tuple_and_array!(11;A,B,C,D,E,F,G,H,I,J,K;K,J,I,H,G,F,E,D,C,B,A);
impl_tuple_and_array!(12;A,B,C,D,E,F,G,H,I,J,K,L;L,K,J,I,H,G,F,E,D,C,B,A);
impl_tuple_and_array!(13;A,B,C,D,E,F,G,H,I,J,K,L,M;M,L,K,J,I,H,G,F,E,D,C,B,A);
impl_tuple_and_array!(14;A,B,C,D,E,F,G,H,I,J,K,L,M,N;N,M,L,K,J,I,H,G,F,E,D,C,B,A);
impl_tuple_and_array!(15;A,B,C,D,E,F,G,H,I,J,K,L,M,N,O;O,N,M,L,K,J,I,H,G,F,E,D,C,B,A);
impl_tuple_and_array!(16;A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P;P,O,N,M,L,K,J,I,H,G,F,E,D,C,B,A);

#[cfg(test)]
mod tests {
    use super::*;
    use stack::UnexpectedTy;

    #[test]
    fn check_tags() {
        let mut stack = Stack::new();

        stack.push(5i64);
        assert_eq!(stack.pop::<i64>(), Ok(5));

        stack.push(11u64);
        assert_eq!(stack.pop::<u64>(), Ok(11));
    }

    #[test]
    fn check_low_memory() {
        let mut stack = Stack::new();

        stack.push(5u64);
        assert_eq!(stack.inner.len(), 9);
    }

    #[test]
    fn check_multi() {
        let mut stack = Stack::new();

        for i in 0..10u64 {
            stack.push(i);
        }
        assert_eq!(stack.inner.len(), 10 * 1 + 10 * 8);

        assert_eq!(
            (0..10).map(|_| stack.pop::<u64>().unwrap()).sum::<u64>(),
            45
        );
        assert_eq!(stack.inner.len(), 0);
    }

    #[test]
    fn check_errors() {
        let mut stack = Stack::new();
        stack.push(true);

        assert_eq!(
            stack.pop::<String>(),
            Err(UnexpectedTy {
                expected: Ty::String,
                got: Ty::Bool,
            }.into())
        );

        stack.push((55i64, false));

        assert_eq!(
            stack.pop::<(i64, u64)>(),
            Err(UnexpectedTy {
                expected: Ty::Uint,
                got: Ty::Bool,
            }.into())
        );

        assert_eq!(
            stack.pop::<(i64, bool, String)>(),
            Err(TupleMismatch {
                expected: 3,
                got: 2,
            }.into())
        );
    }

    #[test]
    fn check_string_mixed() {
        let mut stack = Stack::new();
        stack.push(19i64);
        stack.push("Hello Stack!");
        stack.push(true);

        assert_eq!(stack.inner.len(), 9 + (12 + 8 + 1) + 2);

        assert_eq!(stack.pop(), Ok(true));
        assert_eq!(stack.pop(), Ok("Hello Stack!".to_owned()));
        assert_eq!(stack.pop(), Ok(19i64));
    }

    #[test]
    fn push_array() {
        let mut stack = Stack::new();
        stack.push([5u64, 234u64]);
        assert_eq!(stack.inner.len(), 1 + 1 + 8 + 2 * 8);

        {
            // Make sure `Vec` and array don't differ
            let mut tmp = Stack::new();
            tmp.push(vec![5u64, 234u64]);
            assert_eq!(stack.inner, tmp.inner);
        }

        assert_eq!(stack.pop(), Ok(vec![5u64, 234u64]));
        assert_eq!(stack.inner.len(), 0);
    }

    #[test]
    fn push_tuple() {
        let mut stack = Stack::new();
        stack.push((-5i64, 234u64));
        assert_eq!(stack.inner.len(), 1 + 8 + 1 + 1 + 8 + 8);

        assert_eq!(stack.pop(), Ok((-5i64, 234u64)));
        assert_eq!(stack.inner.len(), 0);
    }

    #[test]
    fn no_invalid_state() {
        let mut stack = Stack::new();
        stack.push(false);
        stack.push(true);

        assert_eq!(
            stack.pop::<String>(),
            Err(UnexpectedTy {
                expected: Ty::String,
                got: Ty::Bool
            }.into())
        );

        // the bools should still be there
        assert_eq!(stack.pop(), Ok(true));
        assert_eq!(stack.pop(), Ok(false));
    }

    #[test]
    fn nested_tuple() {
        let mut stack = Stack::new();
        stack.push((
            "Hello",
            (
                "from a nested tuple",
                "this is ",
                true,
                "ly",
                ("a rather contrived example", 55i64),
            ),
        ));

        assert_eq!(
            stack.pop(),
            Ok((
                "Hello".to_owned(),
                (
                    "from a nested tuple".to_owned(),
                    "this is ".to_owned(),
                    true,
                    "ly".to_owned(),
                    ("a rather contrived example".to_owned(), 55i64)
                )
            ))
        )
    }
}
