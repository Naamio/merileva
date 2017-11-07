use errors::NaamioError;
use futures::Future;
use hyper::Client;
use hyper_rustls::HttpsConnector;
use libc::size_t;
use std::{slice, str};

/// HTTPS client (courtesy of rustls)
pub type HyperClient = Client<HttpsConnector>;
/// The `Future` type used throughout the lib.
pub type NaamioFuture<I> = Box<Future<Item=I, Error=NaamioError>>;
/// A closure which takes a HTTPS client and returns a `Future`. This is
/// how HTTPS client requests are queued in the event loop.
pub type EventLoopRequest = Box<Fn(&HyperClient) -> NaamioFuture<()> + Send + 'static>;

/* JSON */

#[repr(C)]
#[derive(Serialize)]
pub struct RegisterRequest<Str> {
    pub name: Str,
    pub rel_url: Str,
    pub endpoint: Str,
}

#[derive(Deserialize)]
pub struct RegisterResponse {
    pub token: Option<String>,
}

/* FFI */

#[repr(C)]
/// A "borrowed" byte array (lives only as long as the owner).
pub struct ByteArray {
    pub bytes: *const u8,
    pub len: size_t,
}

impl ByteArray {
    pub fn as_str(&self) -> Option<&str> {
        unsafe {
            let byte_slice = slice::from_raw_parts(self.bytes, self.len as usize);
            str::from_utf8(byte_slice).ok()
        }
    }
}

impl<'a> From<&'a str> for ByteArray {
    fn from(s: &'a str) -> ByteArray {
        ByteArray {
            bytes: s.as_ptr(),
            len: s.len() as size_t,
        }
    }
}
