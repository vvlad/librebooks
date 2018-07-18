use core;
use gio;
use glib;

error_chain!{
    foreign_links {
        GLibError(glib::error::BoolError);
        GioError(gio::Error);
        CoreError(core::Error);

    }
}

pub type NoResult = Result<()>;
