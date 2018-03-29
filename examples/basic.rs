extern crate fern;
extern crate log;
extern crate vnodes;

use vnodes::{MapNode, Result, Vnodes};

fn run() -> Result<()> {
    let nodes = Vnodes::new();
    nodes.insert("/foo", -5i64)?;
    nodes.insert("/bar", MapNode::new_node())?;
    nodes.insert("bar/abc", 1u64)?;
    nodes.insert("/bar/5", 55u64)?;

    assert_eq!(nodes.get("/foo"), Ok(-5i64));

    Ok(())
}

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

    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}
