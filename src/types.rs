use errors::NaamioError;
use futures::Future;
use hyper::Client;
use hyper_rustls::HttpsConnector;
use libc::c_char;
use std::str;
use std::ffi::CStr;

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

/// Performs lossy conversion and clones FFI-owned string.
pub fn clone_c_string(p: *const c_char) -> String {
    let s = unsafe { CStr::from_ptr(p) };
    s.to_string_lossy().into_owned()
}
