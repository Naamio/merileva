use {serde_json, utils};
use errors::{NaamioError, NaamioResult};
use futures::{Future, Sink, Stream, future};
use futures::sync::mpsc as futures_mpsc;
use futures::sync::mpsc::Sender as FutureSender;
use hyper::{Body, Client, Method, Request, StatusCode};
use hyper::header::{ContentLength, ContentType, Headers};
use hyper_rustls::HttpsConnector;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value as SerdeValue;
use std::{mem, thread};
use std::sync::Arc;
use tokio_core::reactor::Core;
use types::{EventLoopRequest, HyperClient, NaamioFuture};

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

    fn request_with_request(client: &HyperClient, request: Request)
                           -> NaamioFuture<(StatusCode, Headers, Body)>
    {
        info!("{}: {}", request.method(), request.uri());
        let f = client.request(request).and_then(|mut resp| {
            let code = resp.status();
            info!("Response: {}", code);
            let hdrs = mem::replace(resp.headers_mut(), Headers::new());
            future::ok((code, hdrs, resp.body()))
        }).map_err(NaamioError::from);

        Box::new(f)
    }

    /// Generic request builder for all API requests.
    pub fn request<S, D>(client: &HyperClient, mut req: Request, data: Option<S>)
                        -> NaamioFuture<D>
        where S: Serialize, D: DeserializeOwned + 'static
    {
        if let Some(object) = data {
            req.headers_mut().set(ContentType::json());
            let res = serde_json::to_vec(&object).map(|bytes| {
                debug!("Setting JSON payload");
                req.headers_mut().set(ContentLength(bytes.len() as u64));
                req.set_body::<Vec<u8>>(bytes.into());
            });

            future_try!(res);
        }

        let f = NaamioService::request_with_request(client, req);
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

    #[inline]
    fn queue_closure(&self, f: EventLoopRequest) {
        self.sender.clone().send(f).wait().map_err(|e| {
            error!("Cannot queue request in event loop: {}", e);
        }).ok();
    }

    pub fn queue_request<C, F, S, D>(&self, method: Method, url: &str,
                                     data: Option<S>,
                                     call_before: Option<C>,
                                     call_after: F)
                                    -> NaamioResult<()>
        where F: Fn(D) + Send + Sync + 'static,
              D: DeserializeOwned + 'static,
              S: Serialize + Send + 'static,
              C: Fn(&mut Request) + Send + Sync + 'static
    {
        let callback = Arc::new(call_after);
        let url = utils::parse_url(url)?;

        let closure = Box::new(move |client: &HyperClient| {
            let callback = callback.clone();
            let mut req = Request::new(method.clone(), url.clone());
            if let Some(ref c) = call_before {
                c(&mut req);
            }

            let f = Self::request(client, req, data.as_ref());
            let f = f.and_then(move |resp| {
                (&callback)(resp);
                future::ok(())
            });

            Box::new(f) as NaamioFuture<()>
        });

        self.queue_closure(closure);
        Ok(())
    }
}

impl Drop for NaamioService {
    fn drop(&mut self) {
        info!("Service is being deallocated.");
    }
}
