use gio::{resources_register, Resource};
use glib::Bytes;
use glib::{IsA, Object};
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

macro_rules! define_resource {
    ($($name: ident, $type: ty)+) => {
        $(
            pub fn $name(&self, id: &str) -> $type {
                self.get(id)
            }
        )+
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

    define_resource!(button, gtk::Button);
    define_resource!(image, gtk::Image);
    define_resource!(label, gtk::Label);
}

pub fn main_window() -> Resources {
    Resources({
        let builder = gtk::Builder::new();

        builder
            .add_from_resource(resource!("ui/main_window.ui"))
            .expect("should add ui/main_window.ui");
        builder
    })
}
