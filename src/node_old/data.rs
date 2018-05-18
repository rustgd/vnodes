use std::process::abort;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

#[repr(C)]
pub struct NodeData<T> {
    pub _raw: RawNodeData,
    pub node: T,
    strong: AtomicUsize,
    //weak: AtomicUsize,
}

impl<T> NodeData<T>
where
    T: Node + 'static,
{
    pub fn new(node: T) -> Box<Self> {
        let data = NodeData {
            _raw: RawNodeData {
                action: raw_action_node_data::<T>,
            },
            node,
            strong: AtomicUsize::new(1),
            //weak: AtomicUsize::new(0),
        };

        Box::new(data)
    }
}

unsafe extern "C" fn raw_action_node_data<T>(
    node_data: *mut RawNodeData,
    context: *mut Vnodes,
    action: Action,
    arg: RawValue,
) -> RawValue
where
    T: Node + 'static,
{
    let arg = Value::from_raw(arg);

    trace!(
        "action function called with args {{ node: {:x}, context: {:x}, action: {:?}, arg: {:?} }}",
        node_data as usize,
        context as usize,
        action,
        arg,
    );

    let this = node_data as *const NodeData<T>;
    let this = &*this;

    Value::from_res(action_node_data(this, context, action, arg)).into()
}

unsafe fn action_node_data<'a, T>(
    this: &'a NodeData<T>,
    context: RawContextPtr,
    action: Action,
    arg: Value,
) -> Result<Value<'a>>
where
    T: Node + 'static,
{
    match action {
        Action::Call => unimplemented!(),
        Action::Get => {
            let context = &*context;
            let ident: Interned = ValueConv::from_value(arg)?;

            this.node.get(context, ident)
        }
        Action::Set => {
            let (ident, value): (Interned, Value) = ValueConv::from_value(arg)?;
            let value = value.make_owned();

            this.node.set(&*context, ident, value)?;

            Ok(Value::Void)
        }
        Action::Clone => {
            let old = this.strong.fetch_add(1, Ordering::Relaxed);

            // Refcount is way too high, we got serious memory leaks.
            if old > (!0 >> 1) {
                abort();
            }

            Ok(Value::Void)
        }
        Action::Drop => {
            let old = this.strong.fetch_sub(1, Ordering::Relaxed);

            match old {
                1 => {
                    // Synchronize with other threads to make sure we have unique ownership
                    assert_eq!(this.strong.load(Ordering::Acquire), 0);

                    // Don't rely on type inference here
                    let b: Box<NodeData<T>> =
                        Box::from_raw(this as *const NodeData<T> as *mut NodeData<T>);
                    drop(b);
                }
                _ => {}
            }

            Ok(Value::Void)
        }
        Action::List => unimplemented!(),
    }
}

fn drop_box<T>(b: Box<NodeData<T>>) {
    drop(b);
}
