extern crate nalgebra;
extern crate simplesvg;
extern crate rand;
extern crate getopts;

pub mod geom;
pub mod geom_import;
pub mod trace;
pub mod render;
pub mod parcombs;
pub mod app;

use std::env;
use std::io;
use std::io::Write;

fn main() {
    let params = app::parse_command_line(&(env::args().collect()))
        .unwrap_or_else(|e| { panic!(e) });
    
    if let Err(e) = app::do_run(&params) {
        writeln!(&mut io::stderr(), "{}", e).unwrap();
    }
}