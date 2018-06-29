use gtk::prelude::*;
use librebooks_core::Player;
use std::sync::Arc;
use std::sync::Mutex;

use resources;

macro_rules! clone {
    (@param _ ) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
    ($($n:ident),+ => move |$($p:tt : $z:ty),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p) : $z,)+| $body
        }
    );
}

#[derive(Clone, Copy, Debug)]
pub enum AppCommand {
    TogglePlay,
    SkipBackward,
    SkipForward,
    NextChaper,
    PreviousChapter,
}

#[derive(Clone)]
pub struct AppController {
    resources: resources::Resources,
    player: Player,
}

impl AppController {
    pub fn new() -> Self {
        AppController {
            resources: resources::main_window(),
            player: Player::new(),
        }
    }

    pub fn prepare(&mut self, application: &gtk::Application) {
        self.player.open("/home/vvlad/Projects/Personal/audible/EmpiresofEVEAHistoryoftheGreatWarsofEVEOnlineUnabridged_ep6.mp3");
        let window = self.resources.get::<gtk::ApplicationWindow>("main_window");
        window.set_application(application);
        window.set_title("LibreBooks");
        //window.show_all();
    }

    fn command(&mut self, command: AppCommand) {
        println!("{:?}", command);
    }

    pub fn connect(&self, controller: Arc<Mutex<Self>>) {
        let resources = &self.resources;

        resources
            .button("toggle-play")
            .connect_clicked(clone!(controller => move |_| {
            controller
                .lock()
                .map(|mut controller| controller.command(AppCommand::TogglePlay)).is_ok();
        }));

        resources
            .button("skip-backward")
            .connect_clicked(clone!(controller => move |_| {
            controller
                .lock()
                .map(|mut controller| controller.command(AppCommand::SkipBackward)).is_ok();
        }));

        resources
            .button("skip-forward")
            .connect_clicked(clone!(controller => move |_| {
            controller
                .lock()
                .map(|mut controller| controller.command(AppCommand::SkipForward)).is_ok();
        }));

        resources
            .button("next-chapter")
            .connect_clicked(clone!(controller => move |_| {
            controller
                .lock()
                .map(|mut controller| controller.command(AppCommand::NextChaper)).is_ok();
        }));

        resources
            .button("previous-chapter")
            .connect_clicked(clone!(controller => move |_| {
            controller
                .lock()
                .map(|mut controller| controller.command(AppCommand::PreviousChapter)).is_ok();
        }));

        resources
            .button("chapter")
            .connect_clicked(clone!(controller => move |_| {
            controller
                .lock()
                .map(|mut controller| controller.command(AppCommand::SkipForward)).is_ok();
        }));
    }
}

#[derive(Clone)]
pub struct App {
    controller: Arc<Mutex<AppController>>,
}

impl App {
    pub fn new() -> Self {
        App {
            controller: Arc::new(Mutex::new(AppController::new())),
        }
    }

    pub fn activate(&mut self, application: &gtk::Application) {
        let mut controller = self.controller.lock().unwrap();
        controller.prepare(application);
        controller.connect(self.controller.clone());
    }
}
