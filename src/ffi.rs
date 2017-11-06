use chrono::offset::Utc;
use env_logger::LogBuilder;
use futures::{Future, future};
use hyper::{Method, Uri};
use libc::uint8_t;
use log::{LogRecord, LogLevelFilter};
use service::NaamioService;
use std::sync::Arc;
use types::{ByteArray, HyperClient};
use types::{NaamioFuture, RegisterRequest, RegisterResponse};
use utils::NAAMIO_ADDRESS;

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

/* Exported functions */

pub extern fn set_naamio_host(addr: ByteArray) {
    match addr.as_str() {
        Some(s) if s.parse::<Uri>().is_ok() => {
            info!("Setting Naamio host to {}", s);
            // Note that we can't use normal str<->String functions,
            // because we don't own the value.
            *NAAMIO_ADDRESS.write() = s.chars().collect::<String>();
        },
        _ => error!("Cannot set ")
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
