use errors::NaamioError;
use futures::Future;
use hyper::Client;
use hyper_rustls::HttpsConnector;

/// HTTPS client (courtesy of rustls)
pub type HyperClient = Client<HttpsConnector>;
/// The `Future` type used throughout the lib.
pub type NaamioFuture<I> = Box<Future<Item=I, Error=NaamioError>>;
/// A closure which takes a HTTPS client and returns a `Future`. This is
/// how HTTPS client requests are queued in the event loop.
pub type EventLoopRequest = Box<Fn(&HyperClient) -> NaamioFuture<()> + Send + 'static>;
