#![feature(extern_prelude)]
#![feature(macro_rules)]

#[macro_use]
extern crate error_chain;
extern crate chrono;
extern crate gio;
extern crate glib;
extern crate gtk;

extern crate librebooks_core as core;

use gio::prelude::*;

mod macros;

mod app;
mod errors;

use errors::Result;
mod resources;

quick_main!(run);

fn run() -> Result<()> {
    resources::init()?;
    gtk::init()?;
    core::init()?;

    app::launch()?;

    gtk::main();
    Ok(())
}
