use errors::NaamioError;
use futures::{Future, Stream, future};
use hyper::{Body, Error as HyperError, Headers};
use hyper::header::{ContentLength};
use types::NaamioFuture;

/// Return a `Future` that acquires the accumulated request body.
/// FIXME: Prone to DDoS attack! Restrict content length?
pub fn acquire_body(headers: &Headers, body: Body)
                   -> Box<Future<Item=Vec<u8>, Error=HyperError>> {
    let mut bytes = vec![];
    if let Some(l) = headers.get::<ContentLength>() {
        bytes.reserve(**l as usize);
    }

    let f = body.fold(bytes, |mut acc, ref chunk| {
        acc.extend_from_slice(chunk);
        future::ok::<_, HyperError>(acc)
    });

    Box::new(f)
}

/// ... only to map the `HyperError` with `NaamioError`
#[inline]
pub fn acquire_body_with_err(headers: &Headers, body: Body)
                            -> NaamioFuture<Vec<u8>> {
    let b = acquire_body(headers, body);
    Box::new(b.map_err(NaamioError::from))
}

macro_rules! future_try {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => return Box::new(future::err(e.into()))
        }
    };
}

macro_rules! future_try_box {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => return Box::new(future::err(e.into())) as NaamioFuture<_>
        }
    };
}
