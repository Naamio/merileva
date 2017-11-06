use futures::{Future, future};
use hyper::Method;
use libc::uint8_t;
use service::NaamioService;
use std::sync::Arc;
use types::{ByteArray, HyperClient};
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
            let callback = callback.clone();
            let f = Self::request::<_, RegisterResponse>(client,
                                                         Method::Post,
                                                         "/register",
                                                         Some(data.clone()));
            let f = f.and_then(move |resp| {
                if let Some(s) = resp.token.as_ref() {
                    (&callback)(s.as_str().into());
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
pub extern fn register_plugin(p: *mut NaamioService,
                              req: *mut RegisterRequest<ByteArray>,
                              f: extern fn(ByteArray))
{
    let r = unsafe { &*req };
    match (r.name.as_str(), r.rel_url.as_str(), r.endpoint.as_str()) {
        (Some(name), Some(url), Some(endpoint)) => {
            let service = unsafe { &*p };
            service.register_plugin(name, url, endpoint, move |arr| {
                f(arr);
            })
        },
        _ => error!("Cannot extract string from FFI byte slices"),
    }
}
