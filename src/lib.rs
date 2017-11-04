extern crate futures;
extern crate hyper;
extern crate hyper_rustls;
#[macro_use] extern crate lazy_static;
extern crate libc;
#[macro_use] extern crate log;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

#[macro_use] mod utils;
mod errors;
mod service;
mod types;

use libc::uint8_t;
use service::NaamioService;

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
                              name: String,
                              rel_url: String)
{
    //
}
