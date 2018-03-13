use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub enum BackendError {
    Generic,
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BackendError::Generic => write!(f, "Backend Error"),
        }
    }
}

impl Error for BackendError {
    fn description(&self) -> &str {
        match *self {
            BackendError::Generic => "Backend Error",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            BackendError::Generic => None,
        }
    }
}
