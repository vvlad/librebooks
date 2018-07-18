use std::io;
use std::sync::mpsc;

use player;

use gst;
use serde_json;

error_chain!{
    foreign_links {
        PlayerEventError(mpsc::SendError<player::Event>);
        JsonError(serde_json::Error);
        IOError(io::Error);
        GTSError(gst::Error);
        //PlayerError(mpsc::SendError<player::backend::Command>);
    }
}
