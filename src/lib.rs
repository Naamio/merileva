extern crate futures;
extern crate hyper;
extern crate hyper_rustls;
extern crate libc;
#[macro_use] extern crate log;
extern crate tokio_core;

mod errors;
mod service;
mod types;

use libc::{c_uint, size_t};
use service::NaamioService;
use std::{mem, ptr};

#[no_mangle]
pub extern fn create_service(threads: c_uint) -> size_t {
    let mut service = NaamioService::new(threads as usize);
    let ptr = &mut service as *const _;
    mem::forget(service);       // don't run destructor
    ptr as size_t
}

#[no_mangle]
pub extern fn drop_service(p: size_t) {
    let p = p as *const NaamioService;
    let _service = unsafe { ptr::read(p) };
    println!("Successfully recaptured service!");
}
