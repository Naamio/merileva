use errors::NaamioError;
use futures::Future;
use hyper::Client;
use hyper_rustls::HttpsConnector;
use libc::size_t;

/// HTTPS client (courtesy of rustls)
pub type HyperClient = Client<HttpsConnector>;
/// The `Future` type used throughout the lib.
pub type NaamioFuture<I> = Box<Future<Item=I, Error=NaamioError>>;
/// A closure which takes a HTTPS client and returns a `Future`. This is
/// how HTTPS client requests are queued in the event loop.
pub type EventLoopRequest = Box<Fn(&HyperClient) -> NaamioFuture<()> + Send + 'static>;

/* JSON */

#[derive(Serialize)]
pub struct RegisterRequest<'a> {
    pub name: &'a str,
    pub rel_url: &'a str,
    pub endpoint: &'a str,
}

#[derive(Deserialize)]
pub struct RegisterResponse {
    pub token: Option<String>,
}

/* FFI */

pub struct Opaque;

#[repr(C)]
pub struct ByteArray {      // NOTE: Lives only as long as the owner
    pub bytes: *const u8,
    pub len: size_t,
}

impl<'a> From<&'a str> for ByteArray {
    fn from(s: &'a str) -> ByteArray {
        ByteArray {
            bytes: s.as_ptr(),
            len: s.len() as size_t,
        }
    }
}
