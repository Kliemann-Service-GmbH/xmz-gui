extern crate gtk;
extern crate gdk;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver, RecvError, TryRecvError};
use std::thread;

use gio;
use gio::ActionMapExt;
use gio::ApplicationExt;
use gio::SimpleActionExt;
use glib;
use self::gio::prelude::*;
use self::gtk::prelude::*;
use settings::Settings;

use backend::{Backend, BKCommand, BKResponse};
use backend;


const APP_ID: &'static str = "com.gaswarnanlagen.xmz-gui";


/// Application Controller
///
/// Der AppController steuert die Anwendung.
/// Gleichzeitig hält diese Struktur wichtige Komponenten der Anwendung,
/// wie das Backend, die gtk::Builder-. sowie die gtk::Application-Instanz.
pub struct AppController {
    /// der `gtk::Builder` für Zugriff auf die Elemente aus dem Glade File
    pub gtk_builder: gtk::Builder,
    pub gtk_app: gtk::Application,
    pub backend: Sender<backend::BKCommand>,
    pub internal: Sender<InternalCommand>,

    pub syncing: bool,
    pub server_url: String,

    pub state: AppState,
    pub since: Option<String>,
    settings: Settings,
}

#[derive(Debug, Clone)]
pub enum AppState {
    /// Index View
    Index,
    /// Störung, Wartungszeitraum überschritten
    StoerungWartung,
}

/// Hält eine oder keine `ApplicationController` Instanz
static mut CONTROLLER: Option<Arc<Mutex<AppController>>> = None;

/// Führt eine Funktion des `ApplicationController`s, im glib Context aus.
macro_rules! APPCTL {
    ($fn: ident, ($($x: ident),*) ) => {{
        if let Some(ctx) = glib::MainContext::default() {
            ctx.invoke(move || {
                $( let $x = $x.clone(); )*
                if let Some(op) = AppController::def() {
                    op.lock().unwrap().$fn($($x),*);
                }
            });
        }
    }};
    ($fn: ident) => {{
        APPCTL!($fn, ( ) );
    }}
}

impl AppController {
    /// Diese Funktion wird vom `APPCTL!` Macro verwendet um die `ApplicationController` Instanz
    /// anzusprechen.
    pub fn def() -> Option<Arc<Mutex<AppController>>> {
        unsafe {
            match CONTROLLER {
                Some(ref m) => Some(m.clone()),
                None => None,
            }
        }
    }

    /// Erstellt einen neuen `ApplicationController`
    pub fn new(app: gtk::Application,
            builder: gtk::Builder,
            tx: Sender<BKCommand>,
            itx: Sender<InternalCommand>,
            settings: Settings,
        ) -> AppController {

        AppController {
            gtk_builder: builder,
            gtk_app: app,
            backend: tx,
            internal: itx,

            syncing: false,
            server_url: settings.clone().server.url,

            state: AppState::Index,
            since: None,
            settings,
        }
    }

    /// Aktiviert die Anwendung
    pub fn activate(&self) {
        let window: gtk::Window = self.gtk_builder
            .get_object("main_window")
            .expect("Couldn't find main_window in ui file.");
        if self.settings.fullscreen {
            window.fullscreen();
        }
        window.show();
        window.present();
    }

    pub fn quit(&self) {
        self.disconnect();
        self.gtk_app.quit();
    }

    /// Wechselt den aktuellen Status der Anwendung
    pub fn set_state(&mut self, state: AppState) {
        self.state = state;

        let widget_name = match self.state {
            AppState::Index => "index",
            AppState::StoerungWartung => "stoerung_wartung",
        };

        self.gtk_builder
            .get_object::<gtk::Stack>("stack_main_content")
            .expect("Can't find 'stack_main_content' in ui file.")
            .set_visible_child_name(widget_name);

        // Headerbar
        let bar_name = match self.state {
            _ => "normal",
        };

        self.gtk_builder
            .get_object::<gtk::Stack>("stack_headerbar")
            .expect("Can't find 'stack_headerbar' in ui file.")
            .set_visible_child_name(bar_name);
    }


    pub fn load_more_normal(&mut self) {
    }

    /// Initialer Anwendung Status
    pub fn init(&mut self) {
        // self.set_state(AppState::Index);
        self.set_state(AppState::Index);
        // Versuche Verbindung mit dem Server
        self.connect(None);
    }

    pub fn sync(&mut self) {
        if !self.syncing {
            self.syncing = true;
            self.backend.send(BKCommand::Sync).unwrap();
        }
    }

    pub fn synced(&mut self, since: Option<String>) {
        self.syncing = false;
        self.since = since;
        self.sync();
    }

    pub fn sync_error(&mut self) {
        self.syncing = false;
        self.sync();
    }

    pub fn about_dialog(&self) {
        let window: gtk::ApplicationWindow = self.gtk_builder
            .get_object("window_main")
            .expect("Can't find 'window_main' in ui file.");

        let dialog = gtk::AboutDialog::new();
        dialog.set_logo_icon_name(APP_ID);
        dialog.set_comments("Grafische Benutzeroberfläche der 'xMZ-Plattform'");
        dialog.set_copyright("© 2018 Stefan Müller");
        dialog.set_license_type(gtk::License::Gpl20);
        dialog.set_modal(true);
        dialog.set_version(env!("CARGO_PKG_VERSION"));
        dialog.set_program_name("xmz-gui");
        dialog.set_website("https://gaswarnanlagen.com");
        dialog.set_website_label("Weite Informationen zur Software und der Ra-GAS GmbH");
        dialog.set_transient_for(&window);

        dialog.set_artists(&[
            "Helge Kliemann",
        ]);

        dialog.set_authors(&[
            "Stefan Müller",
        ]);

        dialog.add_credit_section("Name by", &["zzeroo"]);

        dialog.show();
    }

    pub fn show_error(&self, msg: String) {
        let window = self.gtk_builder
            .get_object::<gtk::Window>("window_main")
            .expect("Couldn't find 'window_main' in ui file.");
        let dialog = gtk::MessageDialog::new(Some(&window),
                                            gtk::DialogFlags::MODAL,
                                            gtk::MessageType::Warning,
                                            gtk::ButtonsType::Ok,
                                            &msg);
        dialog.show();
        dialog.connect_response(move |d, _| { d.destroy(); });
    }

    pub fn connect(&mut self, server: Option<String>) -> Option<()> {
        self.server_url = match server {
            Some(s) => s,
            None => String::from("http://0.0.0.0:3000"),
        };

        let url = self.server_url.clone();
        self.backend.send(BKCommand::Connect(url)).unwrap();
        Some(())
    }

    pub fn disconnect(&self) {
        self.backend.send(BKCommand::ShutDown).unwrap();
    }

}

/// Basis Application
///
/// Startet die Anwendung und kontrolliert und steuert die UI.
pub struct App {
    /// Für den Zugriff auf die UI Elemente
    gtk_builder: gtk::Builder,

    op: Arc<Mutex<AppController>>,
}

impl App {
    /// Erzeugt eine neue App Instanz
    pub fn new(settings: Settings) {
        let gtk_app = gtk::Application::new(Some(APP_ID), gio::ApplicationFlags::empty())
            .expect("Failed to initalize GtkApplication");

        gtk_app.set_accels_for_action("app.quit", &["<Ctrl>Q"]);

        gtk_app.connect_startup(move |gtk_app| {
            let (tx, rx): (Sender<BKResponse>, Receiver<BKResponse>) = channel();
            let (itx, irx): (Sender<InternalCommand>, Receiver<InternalCommand>) = channel();

            let bk = Backend::new(tx);
            let apptx = bk.run();

            let gtk_builder = gtk::Builder::new_from_resource("/com/gaswarnanlagen/xmz-gui/main_window.glade");
            let window: gtk::Window = gtk_builder
                .get_object("main_window")
                .expect("Couln't find main_window in ui file.");
            window.set_application(gtk_app);

            let op = Arc::new(Mutex::new(
                AppController::new(gtk_app.clone(), gtk_builder.clone(), apptx, itx, settings.clone())
            ));

            unsafe {
                CONTROLLER = Some(op.clone());
            }

            backend_loop(rx);
            appop_loop(irx);

            let app = App {
                gtk_builder: gtk_builder,
                op: op.clone(),
            };

            gtk_app.connect_activate(move |_| { op.lock().unwrap().activate() });

            app.connect_gtk();
            app.run();
        });

        gtk_app.run(&[]);
    }

    pub fn connect_gtk(&self) {
        // Set up shutdown callback
        let window: gtk::Window = self.gtk_builder
            .get_object("main_window")
            .expect("Couln't find main_window in ui file.");

        window.set_title("xmz-gui");
        window.show_all();

        let op = self.op.clone();
        window.connect_delete_event(move |_, _| {
            op.lock().unwrap().quit();
            Inhibit(false)
        });

        self.create_actions();
    }

    fn create_actions(&self) {
        let settings = gio::SimpleAction::new("settings", None);

        let quit = gio::SimpleAction::new("quit", None);
        let about = gio::SimpleAction::new("about", None);

        let op = &self.op;

        op.lock().unwrap().gtk_app.add_action(&settings);
        op.lock().unwrap().gtk_app.add_action(&quit);
        op.lock().unwrap().gtk_app.add_action(&about);

        quit.connect_activate(clone!(op => move |_, _| op.lock().unwrap().quit() ));
        about.connect_activate(clone!(op => move |_, _| op.lock().unwrap().about_dialog() ));

        settings.connect_activate(move |_, _| { println!("SETTINGS"); });
        settings.set_enabled(false);
    }

    pub fn run(&self) {
        self.op.lock().unwrap().init();

        glib::set_application_name("xmz-gui");
        glib::set_prgname(Some("xmz-gui"));

        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/com/gaswarnanlagen/xmz-gui/app.css");
        gtk::StyleContext::add_provider_for_screen(&gdk::Screen::get_default().unwrap(), &provider, 600);
    }
}

fn backend_loop(rx: Receiver<BKResponse>) {
    thread::spawn(move || {
        let mut shutting_down = false;
        loop {
            let recv = rx.recv();

            if let Err(RecvError) = recv {
                // stopping this backend loop thread
                break;
            }

            if shutting_down {
                // ignore this event, we're about shutting down this thread
                continue;
            }

            match recv {
                Err(RecvError) => { break; },
                Ok(BKResponse::ShutDown) => { shutting_down = true; },

                Ok(BKResponse::ConnectSuccessfull) => {
                    APPCTL!(sync);
                }
                Ok(BKResponse::Sync(since)) => {
                    println!("SYNC");
                    let s = Some(since);
                    APPCTL!(synced, (s));
                }
                Ok(BKResponse::SyncError(_)) => {
                    println!("SYNC Error");
                    APPCTL!(sync_error);
                }
                Ok(err) => {
                    println!("Fehler: verstehe die Backend Antwort nicht: {:?}", err);
                }
            };
        }
    });
}


#[derive(Debug)]
pub enum InternalCommand {
    LoadMoreNormal,
}

fn appop_loop(rx: Receiver<InternalCommand>) {
    thread::spawn(move || {
        loop {
            let recv = rx.recv();
            match recv {
                Ok(InternalCommand::LoadMoreNormal) => {
                    APPCTL!(load_more_normal);
                },
                Err(_) => {
                    break;
                }
            };
        }
    });
}
