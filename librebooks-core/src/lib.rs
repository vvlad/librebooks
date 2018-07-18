#![feature(associated_type_defaults)]
#![feature(duration_as_u128)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate error_chain;

extern crate glib;
extern crate gstreamer as gst;

extern crate gstreamer_player as gst_player;

use std::sync::mpsc;
use std::thread;

mod macros;

mod metadata;
pub use metadata::Metadata;

mod errors;
pub use errors::Error;
use errors::Result;

pub trait CommandChannel {
    type Command;
    type Event;

    fn spawn<F>(f: F) -> (mpsc::Sender<Self::Command>, mpsc::Receiver<Self::Event>)
    where
        <Self as CommandChannel>::Event: Send + 'static,
        <Self as CommandChannel>::Command: Send + 'static,
        F: FnOnce(mpsc::Receiver<Self::Command>, mpsc::Sender<Self::Event>),
        F: Send + 'static,
    {
        let commands = mpsc::channel::<Self::Command>();
        let events = mpsc::channel::<Self::Event>();

        {
            let commands = commands.1;
            let events = events.0;
            thread::spawn(move || f(commands, events));
        }

        (commands.0, events.1)
    }
}

pub fn init() -> Result<()> {
    gst::init()?;
    Ok(())
}

// let backend = backend::channel();

pub mod player {
    use glib::prelude::*;
    use glib::Cast;
    use glib::Value;
    use gst;
    use gst_player;
    pub use gst_player::PlayerState as State;

    use std::path;
    use std::sync::mpsc;
    use std::time;

    pub use gst::ClockTime;
    pub use metadata::{Chapter, Metadata};

    #[derive(Debug)]
    pub enum Event {
        MetadataChanged(Metadata),
        StateChanged(State),
        Progress(time::Duration),
    }

    pub struct Player {
        player: gst_player::Player,
        events: mpsc::Sender<Event>,
    }

    impl Player {
        pub fn new(events: mpsc::Sender<Event>) -> Player {
            let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
            let player = gst_player::Player::new(
                None,
                Some(&dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()),
            );

            player.connect_end_of_stream(clone!(player => move |_| {
                    player.stop();
                }));

            player.connect_error(clone!(player => move |_,_err| {
                    player.stop();
                }));

            player.connect_state_changed(clone!(events => move |_, state| {
                events.send(Event::StateChanged(state)).expect("delivered");
            }));

            player.connect_position_updated(clone!(events => move |_, position| {
                if let Some(nanoseconds) = position.nanoseconds() {
                    events.send(Event::Progress(time::Duration::from_nanos(nanoseconds))).expect("delivered");
                }
            }));

            Player { player, events }
        }

        pub fn open(&self, path: path::PathBuf) {
            let uri = {
                let file = format!("file://{}", path.to_str().unwrap());
                Value::from(&file)
            };

            if let Ok(metadata) = Metadata::from_file(&path) {
                self.events
                    .send(Event::MetadataChanged(metadata))
                    .expect("delivered");
                self.player.set_property("uri", &uri).is_ok();
            }
        }

        pub fn seek(&self, position: time::Duration) {
            self.player
                .seek(ClockTime::from_nseconds(position.as_nanos() as u64));
        }
        pub fn play(&self) {
            println!("Play");
            self.player.play();
        }

        pub fn pause(&self) {
            println!("Pause");
            self.player.pause();
        }
    }

}
