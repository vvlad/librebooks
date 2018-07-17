use gio::{resources_register, Resource};
use glib::Bytes;
use glib::{IsA, Object};
use gtk::prelude::*;
use gtk::BuilderExt;

use errors::NoResult;

pub fn init() -> NoResult {
    let res_bytes = include_bytes!("../res/resources.gresource");

    let gbytes = Bytes::from(res_bytes.as_ref());
    let resource = Resource::new_from_data(&gbytes)?;

    resources_register(&resource);
    Ok(())
}

macro_rules! resource {
    ($path:expr) => {
        &format!("/com/verestiuc/librebooks/{}", $path)
    };
}

#[derive(Clone, Debug)]
pub struct Resources(gtk::Builder);

impl Resources {
    pub fn get<T: IsA<Object>>(&self, id: &str) -> T {
        self.0
            .get_object(id)
            .expect(&format!("Couldn't get {}", id))
    }
}

#[derive(Clone)]
pub struct MainWindow {
    pub view: gtk::ApplicationWindow,
    pub toggle_play: gtk::Button,
    pub skip_forward: gtk::Button,
    pub skip_backward: gtk::Button,
    pub next_chapter: gtk::Button,
    pub previous_chapter: gtk::Button,
    pub played: gtk::Label,
    pub remaining: gtk::Label,
    pub open: gtk::Button,
}

impl MainWindow {
    pub fn new() -> MainWindow {
        let resources = Resources({
            let builder = gtk::Builder::new();

            builder
                .add_from_resource(resource!("ui/main_window.ui"))
                .expect("should add ui/main_window.ui");
            builder
        });

        MainWindow {
            view: resources.get("main-window"),
            toggle_play: resources.get("toggle-play"),
            skip_forward: resources.get("skip-forward"),
            skip_backward: resources.get("skip-backward"),
            next_chapter: resources.get("next-chapter"),
            previous_chapter: resources.get("previous-chapter"),
            open: resources.get("open"),
            played: resources.get("played"),
            remaining: resources.get("remaining"),
        }
    }

    pub fn show(&self) {
        self.view.show_all();
    }
}
