use chrono::offset::Utc;
use env_logger::LogBuilder;
use futures::{Future, future};
use hyper::Method;
use libc::{c_char, c_void, uint8_t};
use log::{LogRecord, LogLevelFilter};
use serde::de::DeserializeOwned;
use serde_json::Value as SerdeValue;
use service::NaamioService;
use std::ffi::CString;
use std::sync::Arc;
use std::ptr::Unique;
use types::{self, HyperClient};
use types::{NaamioFuture, RegisterRequest, RegisterResponse};
use utils::{NAAMIO_ADDRESS, Url};

impl NaamioService {
    pub fn register_plugin<F, D>(&self, req: RegisterRequest<String>,
                                 host: Option<String>, call: F)
        where F: Fn(D) + Send + Sync + 'static,
              D: DeserializeOwned + 'static
    {
        info!("Registering plugin: {} (endpoint: {}, rel_url: {})",
              &req.name, &req.endpoint, &req.rel_url);
        let callback = Arc::new(call);

        let closure = Box::new(move |client: &HyperClient| {
            let data = json!(&req);
            let callback = callback.clone();
            let url = future_try_box!(host.clone()
                                          .map(|s| Url::absolute(s.as_str()))
                                          .unwrap_or(Url::relative("/register")));
            let f = Self::request::<_, D>(client, Method::Post,
                                          url, Some(data));
            let f = f.and_then(move |resp| {
                (&callback)(resp);
                future::ok::<(), _>(())
            });

            Box::new(f) as NaamioFuture<()>
        });

        self.queue_closure(closure);
    }
}

/* Exported functions */

#[no_mangle]
pub extern fn set_naamio_host(addr: *const c_char) {
    let addr = types::clone_c_string(addr);
    match Url::absolute(&addr) {
        Ok(_) => *NAAMIO_ADDRESS.write() = addr.trim_right_matches('/').to_owned(),
        Err(e) => error!("Cannot set Naamio host ({})", e),
    }
}

#[no_mangle]
pub extern fn create_service(threads: uint8_t) -> *mut NaamioService {
    let service = NaamioService::new(threads as usize);
    let ptr = Box::new(service);
    Box::into_raw(ptr)
}

// NOTE: This doesn't support extensive logging (module-level, for example)
#[no_mangle]
pub extern fn set_log_level(level: uint8_t) {
    let mut builder = LogBuilder::new();
    let level = match level {
        0 => LogLevelFilter::Off,
        1 => LogLevelFilter::Error,
        2 => LogLevelFilter::Warn,
        3 => LogLevelFilter::Info,
        4 => LogLevelFilter::Debug,
        5 => LogLevelFilter::Trace,
        _ => return,
    };

    builder.format(|record: &LogRecord| {
        format!("{:?}: {}: {}", Utc::now(), record.level(), record.args())
    }).filter(None, level);

    builder.init().map_err(|e| {
        error!("Cannot initialize logger! ({})", e);
    }).ok();
}

#[no_mangle]
pub extern fn drop_service(p: *mut NaamioService) {
    let _service = unsafe { Box::from_raw(p) };
}

#[no_mangle]
pub extern fn register_plugin(swift_internal: *mut c_void,
                              p: *mut NaamioService,
                              req: *mut RegisterRequest<*const c_char>,
                              f: extern fn(*mut c_void, *const c_char))
{
    let (u, r) = unsafe {
        (Unique::new_unchecked(swift_internal), &*req)
    };

    let r = RegisterRequest {
        name: types::clone_c_string(r.name),
        rel_url: types::clone_c_string(r.rel_url),
        endpoint: types::clone_c_string(r.endpoint),
    };

    let service = unsafe { &*p };
    service.register_plugin(r, None, move |resp: RegisterResponse| {
        if let Some(ref t) = resp.token {
            let string = CString::new(t.as_str()).unwrap();
            f(u.as_ptr(), string.as_ptr());
        } else {
            error!("[plugin reg] Didn't get token for plugin");
        }
    })
}

#[no_mangle]
pub extern fn register_plugin_with_host(swift_internal: *mut c_void,
                                        p: *mut NaamioService,
                                        host: *const c_char,
                                        req: *mut RegisterRequest<*const c_char>,
                                        f: extern fn(*mut c_void))
{
    let host = types::clone_c_string(host);
    let (u, r) = unsafe {
        (Unique::new_unchecked(swift_internal), &*req)
    };

    let r = RegisterRequest {
        name: types::clone_c_string(r.name),
        rel_url: types::clone_c_string(r.rel_url),
        endpoint: types::clone_c_string(r.endpoint),
    };

    let service = unsafe { &*p };
    service.register_plugin(r, Some(host), move |_: SerdeValue| {
        f(u.as_ptr());
    })
}
