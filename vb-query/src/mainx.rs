// #[macro_use]
// extern crate query_engine;
// 
// use std::os::raw::{c_char};
// use std::ffi::{CString, CStr};
// 
// use query_engine::*;
extern crate vb_query;

use vb_query::*;

fn main() {
    hello();
    // let c_str = CString::new("World Meo").unwrap();
    // let c_world: *const c_char = c_str.as_ptr() as *const c_char;
    // let message = rust_greeting(c_world);
    // let s = unsafe { CStr::from_ptr(message) };
    // println!("Hello {}", s.to_str().unwrap());
}
