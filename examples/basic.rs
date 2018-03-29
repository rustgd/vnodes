extern crate fern;
extern crate log;
extern crate vnodes;

use vnodes::{Vnodes, Result};

fn run() -> Result<()> {
    let nodes = Vnodes::new();
    nodes.insert("foo", -5i64)?;

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
