#![feature(associated_type_defaults)]

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate error_chain;

extern crate glib;
extern crate gstreamer as gst;

extern crate gstreamer_player as gst_player;

pub use gst_player::PlayerState;

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

pub mod player {
    use std::path;
    use std::sync::mpsc;
    use std::time;

    use super::CommandChannel;
    pub use super::Metadata;
    use errors::Result;

    use gst_player::PlayerState as State;

    pub enum Command {
        Play,
        Pause,
        Seek(time::Duration),
        Open(path::PathBuf),
    }

    pub enum Event {
        MetadataChanged(Metadata),
    }

    impl CommandChannel for Player {
        type Event = Event;
        type Command = Command;
    }

    pub fn channel() -> (mpsc::Sender<Command>, mpsc::Receiver<Event>) {
        Player::spawn(move |command, event| {
            let mut player = Player::new(event.clone());

            loop {
                match command.recv() {
                    Ok(Command::Play) => {}
                    Ok(Command::Pause) => {}
                    Ok(Command::Open(path)) => {
                        player.open(path).is_ok();
                    }
                    Ok(Command::Seek(_)) => {}
                    Err(_) => {}
                };
            }
        })
    }

    #[derive(Clone)]
    pub struct Player {
        metadata: Option<Metadata>,
        state: State,
        backend: mpsc::Sender<backend::Command>,
        events: mpsc::Sender<Event>,
    }

    impl Player {
        pub fn new(events: mpsc::Sender<Event>) -> Self {
            let backend = backend::channel();

            Player {
                events,
                metadata: None,
                state: State::Stopped,
                backend: backend.0,
            }
        }

        pub fn open(&mut self, path: path::PathBuf) -> Result<Metadata> {
            let metadata = Metadata::from_file(&path)?;
            self.metadata = Some(metadata.clone());
            self.backend.send(backend::Command::Open(path))?;
            self.events.send(Event::MetadataChanged(metadata.clone()))?;
            Ok(metadata)
        }

        pub fn play(&self) {
            self.backend.send(backend::Command::Play).is_ok();
        }

        pub fn set_state(&mut self, state: State) {
            self.state = state;
            println!("new state {:?}", state);
        }
    }
    pub mod backend {

        use super::CommandChannel;
        use super::State;

        use glib;
        use gst::prelude::*;
        use gst_player;
        use std::path;
        use std::sync::mpsc;

        pub enum Command {
            Play,
            Pause,
            Open(path::PathBuf),
        }
        pub enum Event {
            StateChanged(State),
        }

        pub struct Backend {}

        impl CommandChannel for Backend {
            type Event = Event;
            type Command = Command;
        }

        pub fn channel() -> (mpsc::Sender<Command>, mpsc::Receiver<Event>) {
            Backend::spawn(|command, events| {
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

                player.connect_state_changed(move |_, state| {
                    events.send(Event::StateChanged(state)).is_ok();
                });

                loop {
                    match command.recv() {
                        Ok(Command::Play) => {
                            player.play();
                        }
                        Ok(Command::Pause) => {
                            player.pause();
                        }
                        Ok(Command::Open(path)) => {
                            let uri = {
                                let file = format!("file://{}", path.to_str().unwrap());
                                glib::Value::from(&file)
                            };
                            player.set_property("uri", &uri).is_ok();
                            player.play();
                        }
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }
            })
        }

    }

}
