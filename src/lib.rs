extern crate futures;
extern crate hyper;
extern crate hyper_rustls;
#[macro_use] extern crate log;
extern crate tokio_core;

mod errors;
mod service;
mod types;

use service::NaamioService;
use std::mem;

#[no_mangle]
pub extern fn create_service(threads: usize) -> *mut NaamioService {
    let mut service = NaamioService::new(threads);
    let ptr = &mut service as *mut _;
    mem::forget(service);       // don't run destructor
    ptr
}
