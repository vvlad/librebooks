#![feature(extern_prelude)]
#![feature(proc_macro)]

#[macro_use]
extern crate error_chain;
extern crate chrono;
extern crate gio;
extern crate glib;
extern crate gtk;
#[macro_use]
extern crate relm;
extern crate relm_attributes;
#[macro_use]
extern crate relm_derive;

extern crate librebooks_core as core;

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

    //app::launch()?;

    app::run()?;
    Ok(())
}
