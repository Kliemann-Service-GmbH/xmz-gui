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

    /// Wechselt den aktuellen Status der Anwendung
    pub fn set_state(&mut self, state: AppState) {
        self.state = state;

        let widget_name = match self.state {
            AppState::Index => "index",
            AppState::StoerungWartung => "stoerung_wartung",
        };

        self.gtk_builder
            .get_object::<gtk::Stack>("main_content_stack")
            .expect("Can't find main_content_stack in ui file.")
            .set_visible_child_name(widget_name);

    }


    pub fn load_more_normal(&mut self) {
    }

    /// Initialer Anwendung Status
    pub fn init(&mut self) {
        // self.set_state(AppState::Index);
        self.set_state(AppState::Index);
    }

    pub fn quit(&self) {
        self.gtk_app.quit();
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
