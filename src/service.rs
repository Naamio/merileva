use {serde_json, utils};
use errors::NaamioError;
use futures::{Future, Sink, Stream, future};
use futures::sync::mpsc as futures_mpsc;
use futures::sync::mpsc::Sender as FutureSender;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper::header::{ContentType, Headers};
use hyper_rustls::HttpsConnector;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value as SerdeValue;
use std::{env, mem, thread};
use tokio_core::reactor::Core;
use types::{EventLoopRequest, HyperClient, NaamioFuture};

lazy_static! {
    pub static ref NAAMIO_ADDRESS: String =
        env::var("NAAMIO_ADDR").unwrap_or(String::from("http://localhost:8000"));
}

pub struct NaamioService {
    sender: FutureSender<EventLoopRequest>,
}

impl NaamioService {
    pub fn new(threads: usize) -> NaamioService {
        let (tx, rx) = futures_mpsc::channel(0);
        let _ = thread::spawn(move || {
            let mut core = Core::new().expect("event loop creation");
            let handle = core.handle();
            let https = HttpsConnector::new(threads, &handle);
            let client = Client::configure().connector(https).build(&handle);
            info!("Successfully created client with {} worker threads", threads);

            let listen_messages = rx.for_each(|call: EventLoopRequest| {
                call(&client).map_err(|e| {
                    info!("Error resolving closure: {}", e);
                })
            });

            core.run(listen_messages).expect("running event loop");
        });

        // We don't have any use of the handle beyond this. It'll be
        // detached from the parent, and dropped when the process quits.

        NaamioService {
            sender: tx,
        }
    }

    fn prepare_request_for_url(method: Method, rel_url: &str) -> Request {
        let url = format!("{}{}", &*NAAMIO_ADDRESS, rel_url);
        info!("{}: {}", method, url);
        Request::new(method, url.parse().unwrap())
    }

    fn request_with_request(client: &HyperClient, request: Request)
                           -> NaamioFuture<(StatusCode, Headers, Body)>
    {
        let f = client.request(request).and_then(|mut resp| {
            let code = resp.status();
            info!("Response: {}", code);
            let hdrs = mem::replace(resp.headers_mut(), Headers::new());
            future::ok((code, hdrs, resp.body()))
        }).map_err(NaamioError::from);

        Box::new(f)
    }

    /// Generic request builder for all API requests.
    pub fn request<S, D>(client: &HyperClient, method: Method,
                         rel_url: &str, data: Option<S>)
                        -> NaamioFuture<D>
        where S: Serialize, D: DeserializeOwned + 'static
    {
        let mut request = NaamioService::prepare_request_for_url(method, rel_url);
        request.headers_mut().set(ContentType::json());

        if let Some(object) = data {
            let res = serde_json::to_vec(&object).map(|bytes| {   // FIXME: Error?
                debug!("Setting JSON payload");
                request.set_body::<Vec<u8>>(bytes.into());
            });

            future_try!(res);
        }

        let f = NaamioService::request_with_request(client, request);
        let f = f.and_then(|(code, headers, body)| {
            utils::acquire_body_with_err(&headers, body).and_then(move |vec| {
                if code.is_success() {
                    let res = serde_json::from_slice::<D>(&vec)
                                         .map_err(NaamioError::from);
                    future::result(res)
                } else {
                    let res = serde_json::from_slice::<SerdeValue>(&vec)
                                         .map_err(NaamioError::from);
                    let msg = format!("Response: {:?}", res);
                    future::err(NaamioError::Other(msg))
                }
            })
        });

        Box::new(f)
    }

    pub fn queue_closure(&self, f: EventLoopRequest) {
        self.sender.clone().send(f).wait().map_err(|e| {
            error!("Cannot queue request in event loop: {}", e);
        }).ok();
    }
}

impl Drop for NaamioService {
    fn drop(&mut self) {
        info!("Service is being deallocated.");
    }
}
