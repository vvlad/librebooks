#![feature(extern_prelude)]
#[macro_use]
extern crate error_chain;
extern crate gio;
extern crate glib;
extern crate gtk;
#[macro_use]
extern crate lazy_static;
extern crate librebooks_core;

use gio::prelude::*;
use gtk::prelude::*;
use std::sync::Mutex;

mod app;
mod errors;
use errors::NoResult;
mod resources;

quick_main!(run);

fn run() -> NoResult {
    resources::init()?;

    let gtk_application = gtk::Application::new(
        Some("com.verestiuc.librebooks"),
        gio::ApplicationFlags::empty(),
    )?;

    let app = Mutex::new(app::App::new());

    gtk_application.connect_startup(move |application| {
        app.lock().unwrap().activate(application);
    });

    gtk_application.connect_activate(move |_| {});

    gtk_application.run(&[]);

    Ok(())
}
