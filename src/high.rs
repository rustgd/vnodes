pub trait Node {
    fn call(&self, &[])
}

pub enum Value {
    Signed(i64),
    Unsigned(u64),
}
