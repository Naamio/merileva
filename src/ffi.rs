use chrono::offset::Utc;
use env_logger::LogBuilder;
use hyper::{Method, Request};
use hyper::header::{Authorization, Bearer};
use libc::{c_void, uint8_t};
use log::{LogRecord, LogLevelFilter};
use service::NaamioService;
use std::ffi::CString;
use std::ptr::Unique;
use types::{self, CStrPtr, RegistrationData};
use types::{RequestRequirements, RegistrationResponse};

/* Exported functions */

#[no_mangle]
pub extern fn create_service(threads: uint8_t) -> *mut NaamioService {
    let service = NaamioService::new(threads as usize);
    let ptr = Box::new(service);
    Box::into_raw(ptr)
}

// NOTE: This doesn't support all logging features (module-level filter, for example)
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
                              req: *mut RequestRequirements<CStrPtr>,
                              data: *mut RegistrationData<CStrPtr>,
                              f: extern fn(*mut c_void, CStrPtr))
{
    let (u, r, d) = unsafe {
        (Unique::new_unchecked(swift_internal), &*req, &*data)
    };

    let d: RegistrationData<String> = d.into();
    info!("Registering plugin: {} (endpoint: {}, rel_url: {})",
          &d.name, &d.endpoint, &d.rel_url);

    let service = unsafe { &*p };
    let closure = move |resp: RegistrationResponse| {
        if let Some(ref t) = resp.token {
            let string = CString::new(t.as_str()).unwrap();
            f(u.as_ptr(), string.as_ptr());
        } else {
            error!("Didn't get token for plugin");
        }
    };

    let url = types::clone_c_string(r.url);
    let token = types::clone_c_string(r.token);
    let modifier = move |req: &mut Request| {
        req.headers_mut().set(Authorization(Bearer {
            token: token.clone(),
        }));
    };

    if let Err(e) = service.queue_request(Method::Post, &url,
                                          Some(d), Some(modifier), closure) {
        error!("Cannot register plugin: {}", e);
    }
}
