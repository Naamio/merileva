use futures::{Future, Stream};
use futures::sync::mpsc as futures_mpsc;
use futures::sync::mpsc::Sender as FutureSender;
use hyper::Client;
use hyper_rustls::HttpsConnector;
use std::thread;
use tokio_core::reactor::Core;
use types::EventLoopRequest;

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
}

impl Drop for NaamioService {
    fn drop(&mut self) {
        info!("Service is being deallocated.");
    }
}
