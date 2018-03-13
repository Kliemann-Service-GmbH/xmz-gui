extern crate gtk;
extern crate gdk;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver, RecvError, TryRecvError};
use std::thread;

use gio::ApplicationExt;
use gio::SimpleActionExt;
use gio::ActionMapExt;
use glib;
use gio;
use self::gio::prelude::*;
use self::gtk::prelude::*;

use backend::{Backend, BKCommand, BKResponse};
use backend;


const APP_ID: &'static str = "com.gaswarnanlagen.xmz-gui";


pub struct AppOp {
    pub gtk_builder: gtk::Builder,
    pub gtk_app: gtk::Application,
    pub backend: Sender<backend::BKCommand>,
    pub internal: Sender<InternalCommand>,

    pub state: AppState,
}

#[derive(Debug, Clone)]
pub enum AppState {
    Index,
}

static mut OP: Option<Arc<Mutex<AppOp>>> = None;

macro_rules! APPOP {
    ($fn: ident, ($($x: ident),*) ) => {{
        if let Some(ctx) = glib::MainContext::default() {
            ctx.invoke(move || {
                $( let $x = $x.clone(); )*
                if let Some(op) = AppOp::def() {
                    op.lock().unwrap().$fn($($x),*);
                }
            });
        }
    }};
    ($fn: ident) => {{
        APPOP!($fn, ( ) );
    }}
}

impl AppOp {
    pub fn def() -> Option<Arc<Mutex<AppOp>>> {
        unsafe {
            match OP {
                Some(ref m) => Some(m.clone()),
                None => None,
            }
        }
    }

    pub fn new(app: gtk::Application,
            builder: gtk::Builder,
            tx: Sender<BKCommand>,
            itx: Sender<InternalCommand>) -> AppOp {

        AppOp {
            gtk_builder: builder,
            gtk_app: app,
            backend: tx,
            internal: itx,

            state: AppState::Index,
        }
    }

    pub fn activate(&self) {
        let window: gtk::Window = self.gtk_builder
            .get_object("main_window")
            .expect("Couldn't find main_window in ui file.");
        window.show();
        window.present();
    }

    /// Wechselt den aktuellen Status der Anwendung
    pub fn set_state(&mut self, state: AppState) {
        self.state = state;
    }

    pub fn load_more_normal(&mut self) {
    }

    pub fn quit(&self) {
        self.gtk_app.quit();
    }

    pub fn init(&mut self) {
        self.set_state(AppState::Index);
    }
}

/// Basis Application
///
/// Startet die Anwendung und kontrolliert und steuert die UI.
pub struct App {
    /// Für den Zugriff auf die UI Elemente
    gtk_builder: gtk::Builder,

    op: Arc<Mutex<AppOp>>,
}

impl App {
    /// Erzeugt eine neue App Instanz
    pub fn new() {
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
                AppOp::new(gtk_app.clone(), gtk_builder.clone(), apptx, itx)
            ));

            unsafe {
                OP = Some(op.clone());
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
                    APPOP!(load_more_normal);
                },
                Err(_) => {
                    break;
                }
            };
        }
    });
}