#[macro_use]
extern crate tracing;
#[macro_use]
extern crate rust_embed;

pub mod error;

pub use error::*;
use std::{convert::TryFrom, error::Error, net::SocketAddr, process};
use std::os::raw::{c_char};
use std::ffi::{CString, CStr};

#[no_mangle]
pub extern fn rust_greeting(to: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(to) };
    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string
    };
    CString::new("Hello ".to_owned() + recipient).unwrap().into_raw()
}

type AnyError = Box<dyn Error + Send + Sync + 'static>;

#[no_mangle]
pub fn request() -> Result<(), AnyError> {
    println!("Hello");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn request_test() {
        assert_eq!(true, true);
    }
}
