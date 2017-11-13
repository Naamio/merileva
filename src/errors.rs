use hyper::Error as HyperError;
use hyper::error::UriError;
use serde_json::error::Error as SerdeError;
use std::error::Error;
use std::fmt::{self, Display};
use std::io::Error as IoError;

pub type NaamioResult<T> = Result<T, NaamioError>;

#[derive(Debug)]
pub enum NaamioError {
    Io(IoError),
    Hyper(HyperError),
    Serde(SerdeError),
    Uri(UriError),
    Other(String),
}

impl Display for NaamioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            NaamioError::Io(ref e)      => write!(f, "I/O error: {}", e),
            NaamioError::Hyper(ref e)   => write!(f, "Hyper error: {}", e),
            NaamioError::Serde(ref e)   => write!(f, "Serde error: {}", e),
            NaamioError::Uri(ref e)     => write!(f, "URI error: {}", e),
            NaamioError::Other(ref e)   => write!(f, "Unknown error: {}", e),
        }
    }
}

impl Error for NaamioError {
    fn description(&self) -> &str {
        "NaamioError"
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            NaamioError::Io(ref e)      => Some(e),
            NaamioError::Hyper(ref e)   => Some(e),
            NaamioError::Serde(ref e)   => Some(e),
            NaamioError::Uri(ref e)     => Some(e),
            _ => None,
        }
    }
}

macro_rules! impl_error {
    ($err:ty => $ident:ident) => {
        impl From<$err> for NaamioError {
            fn from(e: $err) -> NaamioError {
                NaamioError::$ident(e)
            }
        }
    }
}

impl_error!(IoError => Io);
impl_error!(HyperError => Hyper);
impl_error!(SerdeError => Serde);
impl_error!(UriError => Uri);
