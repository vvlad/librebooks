use std::ops::{Add, Sub};
use std::sync::mpsc;
use std::thread;
use std::time;

use relm::{Channel, Relm, Update, Widget};

use glib::translate::FromGlib;
use gtk::prelude::*;

use core::player;
use errors::Result;
use resources;

pub struct Model {}

#[derive(Msg, Debug)]
pub enum Msg {
    Quit,
    Open,
    TogglePlay,
    NextChapter,
    PreviousChaper,
    SkipForward,
    SkipBackward,
    PlayerEvent(player::Event),
    ShowChapters,
}

pub struct Application {
    resources: resources::MainWindow,
    player: player::Player,
    state: player::State,
    metadata: player::Metadata,
    position: time::Duration,
    chapters: gtk::Popover,
}

use chrono::prelude::*;
use core::player::State::*;

pub fn formatted_date(dt: chrono::DateTime<Utc>) -> String {
    format!(
        "<span font_family=\"monospace\">{}</span>",
        dt.format("%H:%M:%S")
    )
}

enum SeekDirection {
    Forward(time::Duration),
    Backward(time::Duration),
    At(time::Duration),
}

impl Application {
    fn switch_book(&mut self, metadata: player::Metadata) {
        self.resources.title.set_text(&metadata.title);
        self.metadata = metadata;
        self.player.play();

        for child in self.chapters.get_children().iter() {
            self.chapters.remove(child);
        }
        let vbox = gtk::ListBox::new();

        for chapter in self.metadata.chapters.iter() {
            let label = gtk::Button::new_with_label(&chapter.title);
            vbox.add(&label);
        }

        self.chapters.add(&vbox);
    }

    fn next_chapter(&mut self) {
        if let Some(chapter) = self.chapter_at(self.position.clone()) {
            self.seek(SeekDirection::At(chapter.end));
        }
    }

    fn previous_chaper(&mut self) {
        if let Some(chapter) = self.chapter_at(self.position.clone()) {
            if self.position.as_secs() - chapter.start.as_secs() < 10 {
                if let Some(chapter) =
                    self.chapter_at(chapter.start - time::Duration::from_secs(10))
                {
                    self.seek(SeekDirection::At(chapter.start));
                }
            } else {
                self.seek(SeekDirection::At(chapter.start));
            }
        }
    }

    fn seek(&mut self, seek: SeekDirection) {
        let position = match seek {
            SeekDirection::Forward(delta) => self.position.add(delta),
            SeekDirection::Backward(delta) => self.position.sub(delta),
            SeekDirection::At(position) => position.clone(),
        };
        //self.chapters.append(());
        self.player.seek(position);
    }

    fn update_progress(&mut self, clock: time::Duration) {
        self.position = clock.clone();

        if let Some(chapter) = self.chapter_at(clock) {
            let position = (clock - chapter.start).as_secs() as f64;
            let total = (chapter.end - chapter.start).as_secs() as f64;
            let fraction = position / total;

            self.resources.progress.set_fraction(fraction);
            let dt = Utc.timestamp(position as i64, 0);

            self.resources.played.set_markup(&formatted_date(dt));

            let dt = Utc.timestamp((total - position) as i64, 0);

            self.resources.remaining.set_markup(&formatted_date(dt));
            self.resources.chapter.set_label(&chapter.title);
        }
    }

    fn chapter_at(&self, position: time::Duration) -> Option<player::Chapter> {
        self.metadata
            .chapters
            .iter()
            .find(|chapter| chapter.start <= position && position <= chapter.end)
            .map(|chapter| chapter.clone())
    }

    fn toggle_play(&mut self) {
        match self.state {
            Playing => self.player.pause(),
            Paused => self.player.play(),
            Stopped => self.open(),
            _ => {}
        }
    }

    fn reflect_on_state(&mut self, state: player::State) {
        self.state = state;
    }

    fn open(&mut self) {
        let file_chooser = gtk::FileChooserDialog::with_buttons(
            Some("Open a media file"),
            Some(&self.resources.view),
            gtk::FileChooserAction::Open,
            &[
                ("Cancel", gtk::ResponseType::Cancel),
                ("Open", gtk::ResponseType::Accept),
            ],
        );

        let filter = gtk::FileFilter::new();
        filter.add_mime_type("audio/mpeg");
        file_chooser.add_filter(&filter);

        if gtk::ResponseType::from_glib(file_chooser.run()) == gtk::ResponseType::Accept {
            if let Some(path) = file_chooser.get_filename() {
                self.player.open(path);
            }
        }

        file_chooser.close();
    }

    fn connect(&self, relm: &Relm<Self>) {
        let resources = &self.resources;
        connect!(
            relm,
            resources.view,
            connect_delete_event(_, _),
            return (Some(Msg::Quit), Inhibit(false))
        );

        connect!(
            relm,
            resources.toggle_play,
            connect_clicked(_),
            Msg::TogglePlay
        );

        connect!(
            relm,
            resources.skip_backward,
            connect_clicked(_),
            Msg::SkipBackward
        );

        connect!(
            relm,
            resources.skip_forward,
            connect_clicked(_),
            Msg::SkipForward
        );

        connect!(
            relm,
            resources.next_chapter,
            connect_clicked(_),
            Msg::NextChapter
        );

        connect!(
            relm,
            resources.previous_chapter,
            connect_clicked(_),
            Msg::PreviousChaper
        );

        connect!(
            relm,
            resources.chapter,
            connect_clicked(_),
            Msg::ShowChapters
        );

        connect!(relm, resources.open, connect_clicked(_), Msg::Open);
    }

    fn build_player(relm: &Relm<Self>) -> player::Player {
        let (tx, events) = mpsc::channel();
        let player = player::Player::new(tx);
        let stream = relm.stream().clone();

        let (_channel, sender) = Channel::new(move |event| {
            stream.emit(Msg::PlayerEvent(event));
        });

        thread::spawn(move || loop {
            match events.recv() {
                Ok(event) => {
                    sender.send(event).is_ok();
                }
                Err(_) => {}
            };
        });
        player
    }
}

impl Update for Application {
    type Model = Model;
    type ModelParam = ();
    type Msg = Msg;

    fn model(_: &Relm<Self>, _: ()) -> Model {
        Model {}
    }

    fn update(&mut self, event: Msg) {
        match event {
            Msg::Quit => gtk::main_quit(),
            Msg::Open => self.open(),
            Msg::TogglePlay => self.toggle_play(),
            Msg::SkipForward => self.seek(SeekDirection::Forward(time::Duration::from_secs(10))),
            Msg::SkipBackward => self.seek(SeekDirection::Backward(time::Duration::from_secs(10))),
            Msg::NextChapter => self.next_chapter(),
            Msg::PreviousChaper => self.previous_chaper(),
            Msg::ShowChapters => {
                self.chapters.popup();
            }
            Msg::PlayerEvent(event) => {
                use self::player::Event::*;
                match event {
                    MetadataChanged(metadata) => self.switch_book(metadata),
                    StateChanged(state) => self.reflect_on_state(state),
                    Progress(clock) => self.update_progress(clock),
                };
            }
        }
    }
}

impl Widget for Application {
    type Root = gtk::ApplicationWindow;

    fn root(&self) -> Self::Root {
        self.resources.view.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let resources = resources::MainWindow::new();

        resources.view.show_all();
        let chapters = gtk::Popover::new(Some(&resources.chapter));

        let app = Application {
            player: Self::build_player(relm),
            resources: resources,
            state: player::State::Stopped,
            metadata: Default::default(),
            position: time::Duration::from_secs(0),
            chapters,
        };

        app.connect(relm);
        app
    }
}

pub fn run() -> Result<()> {
    if let Err(_) = Application::run(()) {
        bail!("application run error");
    }
    Ok(())
}
