use config::{Config, ConfigError, Environment, File};


/// Konfiguration der GUI
///
/// Die Settings werden mit dem `config` crate gebildet.
#[derive(Clone)]
#[derive(Debug, Deserialize)]
pub struct Settings {
    // debug: bool,
    pub fullscreen: bool,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Add in settings from the environment (with a prefix of XMZ)
        // Eg.. `XMZ_DEBUG=1 ./target/app` would set the `debug` key
        s.merge(Environment::with_prefix("XMZ"))?;
        s.merge(File::with_name("/boot/xmz-gui.toml").required(false))?;
        s.merge(File::with_name("xmz-gui.toml").required(false))?;

        s.try_into()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let settings = Settings::new();
        assert!(settings.is_ok());
    }

    #[test]
    fn fullscreen() {
        let settings = Settings::new().unwrap();
        assert_eq!(settings.fullscreen, false);
    }
}
