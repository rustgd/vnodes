extern crate fern;
extern crate log;
extern crate vnodes;

use vnodes::{Interned, Value, Vnodes};

fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let nodes = Vnodes::new();
    nodes.insert(Interned::from("foo"), Value::Signed(-5));

    assert_eq!(nodes.get(Interned::from("foo")), Value::Signed(-5));
}
