use gtk::prelude::*;

use core::{player, CommandChannel};

use errors::Result;

mod controller {

    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::mpsc;

    use glib::translate::FromGlib;
    use gtk::prelude::*;

    use chrono::prelude::*;
    use core::{player, CommandChannel};
    use resources::MainWindow;
    use std::borrow::BorrowMut;

    pub struct Controller {
        window: gtk::ApplicationWindow,
        player: mpsc::Sender<player::Command>,
        remainig: gtk::Label,
        metadata: Option<player::Metadata>,
    }

    pub enum Event {}
    pub enum Command {}

    impl CommandChannel for Controller {
        type Event = Event;
        type Command = Command;
    }

    impl Controller {
        pub fn new(player: mpsc::Sender<player::Command>, resources: MainWindow) -> Controller {
            Controller {
                window: resources.view,
                player: player,
                remainig: resources.remaining,
                metadata: None,
            }
        }

        pub fn activate(&self) {
            self.window.show();
        }

        fn select_media(&self) {
            let file_chooser = gtk::FileChooserDialog::with_buttons(
                Some("Open a media file"),
                Some(&self.window),
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
                self.player
                    .send(player::Command::Open(file_chooser.get_filename().unwrap()))
                    .is_ok();
            }
            file_chooser.close();
        }

        pub fn metadata_changed(&mut self, metadata: player::Metadata) {
            self.window
                .set_title(&human_title(&metadata.title.clone(), 30));
            let time = Utc.timestamp(metadata.duration.as_secs() as i64, 0);
            self.remainig
                .set_label(&time.format("-%H:%M:%S").to_string());
            self.metadata = Some(metadata);
            self.play()
        }

        pub fn play(&self) {}
    }

    fn human_title(title: &str, len: usize) -> String {
        if title.len() > len {
            format!("{}...", title[0..len - 3].to_string())
        } else {
            title.to_string()
        }
    }

    thread_local! (
        static GLOBAL: RefCell<Option<Controller>> = RefCell::new(None)
    );

    macro_rules! controller {
        ($block:expr) => {
            GLOBAL.with(move |global| {
                //*global.borrow_mut() = Some(controller);
                if let Some(controller) = (*global.borrow_mut()).take() {
                    $block(controller);
                    *global.borrow_mut() = Some(controller);
                }
            });
        };
    }

    pub fn initialize() {
        let resources = MainWindow::new();
        let (player_command, player_events) = player::channel();
        let controller = Controller::new(player_command, resources.clone());

        GLOBAL.with(move |global| {
            *global.borrow_mut() = Some(controller);
        });

        connect(resources);
    }

    pub fn connect(resources: MainWindow) {
        resources.open.connect_clicked(move |_| {
            controller!(|controller: Controller| {
                controller.select_media();
            });
        });

        resources.view.connect_destroy(move |_| {
            gtk::main_quit();
        });
    }

}

pub fn launch() -> Result<()> {
    controller::initialize();
    // thread::spawn(move || loop {
    //     match player_events.recv() {
    //         Ok(player::Event::MetadataChanged(metadata)) => {
    //             glib::idle_add(|| {
    //                 controller.borrow_mut().metadata_changed(metadata);
    //                 glib::Continue(false)
    //             });
    //         }
    //         Err(_) => {}
    //     }
    // });
    // let this = Arc::new(&controller);

    // controller::Controller::spawn(move |_command, _event| {

    // });

    Ok(())
}
