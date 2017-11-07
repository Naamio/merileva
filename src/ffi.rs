use chrono::offset::Utc;
use env_logger::LogBuilder;
use futures::{Future, future};
use hyper::Method;
use libc::{c_void, uint8_t};
use log::{LogRecord, LogLevelFilter};
use serde::de::DeserializeOwned;
use serde_json::Value as SerdeValue;
use service::NaamioService;
use std::sync::Arc;
use std::ptr::Unique;
use types::{ByteArray, HyperClient};
use types::{NaamioFuture, RegisterRequest, RegisterResponse};
use utils::{NAAMIO_ADDRESS, Url};

impl NaamioService {
    pub fn register_plugin<F, D>(&self, name: &str,
                                 rel_url: &str, endpoint: &str,
                                 host: Option<String>, call: F)
        where F: Fn(D) + Send + Sync + 'static,
              D: DeserializeOwned + 'static
    {
        let data = json!(RegisterRequest { name, rel_url, endpoint });
        info!("Registering plugin: {} (endpoint: {}, rel_url: {})",
              name, endpoint, rel_url);
        let callback = Arc::new(call);

        let closure = Box::new(move |client: &HyperClient| {
            let callback = callback.clone();
            let url = future_try_box!(host.clone()
                                          .map(|s| Url::absolute(s.as_str()))
                                          .unwrap_or(Url::relative("/register")));
            let f = Self::request::<_, D>(client, Method::Post,
                                          url, Some(data.clone()));
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
pub extern fn set_naamio_host(addr: ByteArray) {
    match addr.to_owned_str() {
        Some(ref s) if Url::absolute(&s).is_ok() => {
            info!("Setting Naamio host to {}", s);
            // Note that we can't use normal `String::from` or `to_owned`
            // because we don't own the value.
            *NAAMIO_ADDRESS.write() = s.trim_right_matches('/').to_owned();
        },
        _ => error!("Cannot set Naamio host (Invalid data)"),
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
                              req: *mut RegisterRequest<ByteArray>,
                              f: extern fn(*mut c_void, ByteArray))
{
    let (u, r) = unsafe {
        (Unique::new_unchecked(swift_internal), &*req)
    };

    match (r.name.as_str(), r.rel_url.as_str(), r.endpoint.as_str()) {
        (Some(name), Some(url), Some(endpoint)) => {
            let service = unsafe { &*p };
            service.register_plugin(name, url,
                                    endpoint, None,
                                    move |resp: RegisterResponse| {
                if let Some(t) = resp.token.as_ref() {
                    f(u.as_ptr(), t.as_str().into());
                } else {
                    error!("[plugin reg] Didn't get token for plugin");
                }
            })
        },
        _ => error!("[plugin reg] Cannot extract strings from FFI byte slices"),
    }
}

#[no_mangle]
pub extern fn register_plugin_with_host(swift_internal: *mut c_void,
                                        p: *mut NaamioService,
                                        host: ByteArray,
                                        req: *mut RegisterRequest<ByteArray>,
                                        f: extern fn(*mut c_void))
{
    let (u, r) = unsafe {
        (Unique::new_unchecked(swift_internal), &*req)
    };

    let host = match host.to_owned_str() {
        Some(s) => s,
        None => return,
    };

    match (r.name.as_str(), r.rel_url.as_str(), r.endpoint.as_str()) {
        (Some(name), Some(url), Some(endpoint)) => {
            let service = unsafe { &*p };
            service.register_plugin(name, url,
                                    endpoint, Some(host),
                                    move |_: SerdeValue| {
                f(u.as_ptr());
            })
        },
        _ => error!("[subplugin reg] Cannot extract string from FFI byte slices"),
    }
}
