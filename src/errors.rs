use std::error::Error;
use std::fmt::{self, Display};
use std::io::Error as IoError;

pub type NaamioResult<T> = Result<T, NaamioError>;

#[derive(Debug)]
pub enum NaamioError {
    Io(IoError),
}

impl Display for NaamioError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            NaamioError::Io(ref e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl Error for NaamioError {
    fn description(&self) -> &str {
        "NaamioError"
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            NaamioError::Io(ref e) => Some(e),
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
