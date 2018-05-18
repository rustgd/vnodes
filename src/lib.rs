#![allow(dead_code)]

//#[macro_use]
//extern crate bitflags;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate failure;
extern crate fxhash;
#[macro_use]
extern crate log;
extern crate parking_lot;
extern crate smallvec;

mod macros;

//pub mod raw;

//mod conv;
//mod data;
//mod error;
//mod fs;
mod intern;
//mod map;
mod node;
mod stack;
mod util;
