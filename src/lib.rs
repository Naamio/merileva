extern crate futures;
extern crate hyper;
extern crate hyper_rustls;
extern crate libc;
#[macro_use] extern crate log;
extern crate tokio_core;

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
