use futures::{Future, future};
use hyper::Method;
use libc::uint8_t;
use service::NaamioService;
use std::sync::Arc;
use types::{ByteArray, HyperClient, Opaque};
use types::{NaamioFuture, RegisterRequest, RegisterResponse};

impl NaamioService {
    pub fn register_plugin<F>(&self, name: &str,
                              rel_url: &str, endpoint: &str,
                              call: F)
        where F: Fn(ByteArray) + Send + Sync + 'static
    {
        let data = json!(RegisterRequest { name, rel_url, endpoint });
        info!("plugin {} (endpoint: {}, rel_url: {})", name, endpoint, rel_url);
        let callback = Arc::new(call);

        let closure = Box::new(move |client: &HyperClient| {
            let f = Self::request::<_, RegisterResponse>(client,
                                                         Method::Post,
                                                         "/register",
                                                         Some(data.clone()));
            let callback = callback.clone();
            let f = f.and_then(move |resp| {
                let callback = callback.clone();
                if let Some(s) = resp.token.as_ref() {
                    callback(s.as_str().into());
                } else {
                    error!("Didn't get token for plugin!");
                }

                future::ok::<(), _>(())
            });

            Box::new(f) as NaamioFuture<()>
        });

        self.queue_closure(closure);
    }
}

#[no_mangle]
pub extern fn create_service(threads: uint8_t) -> *mut NaamioService {
    let service = NaamioService::new(threads as usize);
    let ptr = Box::new(service);
    Box::into_raw(ptr)
}

#[no_mangle]
pub extern fn drop_service(p: *mut NaamioService) {
    let _service = unsafe { Box::from_raw(p) };
}

#[no_mangle]
pub extern fn call(ptr: *mut Opaque, f: extern fn(*mut Opaque, ByteArray)) {
    println!("Calling");
    let s = "Hello world!";
    f(ptr, s.into());
    println!("Done!");
}
