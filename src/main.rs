#![deny(unused_extern_crates)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://zzeroo.github.io/share/zzeroo-logo.png",
       html_favicon_url = "https://zzeroo.github.io/share/favicon.ico",
       html_root_url = "https://gaswarnanlagen.com/")]
//! Grafische Oberfläche der **xMZ-Plattform**
//!
//! |||
//! |:---|:------|
//! |**master:**|[![Build Status](https://travis-ci.org/Kliemann-Service-GmbH/xmz-gui.svg?branch=master)](https://travis-ci.org/Kliemann-Service-GmbH/xmz-gui)&nbsp;[![Code Coverage](https://codecov.io/gh/Kliemann-Service-GmbH/xmz-gui/branch/master/graph/badge.svg)](https://codecov.io/gh/Kliemann-Service-GmbH/xmz-gui)|
//! |**development:**|[![Build Status](https://travis-ci.org/Kliemann-Service-GmbH/xmz-gui.svg?branch=development)](https://travis-ci.org/Kliemann-Service-GmbH/xmz-gui)&nbsp;[![Code Coverage](https://codecov.io/gh/Kliemann-Service-GmbH/xmz-gui/branch/development/graph/badge.svg)](https://codecov.io/gh/Kliemann-Service-GmbH/xmz-gui)|
//!
//! Die grafische Oberfläche visualisiert ein `xmz-server`.
//!
//! * **Dokumentation:** [https://kliemann-service-gmbh.github.io/xmz-gui](https://kliemann-service-gmbh.github.io/xmz-gui)
//! * **Quellcode:** [https://github.com/Kliemann-Service-GmbH/xmz-gui](https://github.com/Kliemann-Service-GmbH/xmz-gui)
//!

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate xmz_client;
extern crate config;
extern crate gio;
extern crate glib;
// extern crate serde;

#[macro_use] mod util;
mod app;
mod settings;
mod static_resources;

use app::App;
use settings::Settings;
use xmz_client::{backend, types};


fn main() {
    static_resources::init().expect("GResource initalisation failed.");
    let settings = Settings::new().expect("Kein Konfigurationsdatei gefunden.");
    App::new(settings);
}
