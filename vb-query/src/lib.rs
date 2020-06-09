extern crate query_engine;

use query_engine::*;

use std::os::raw::{c_char};
use std::ffi::{CString, CStr};

#[no_mangle]
pub extern fn query(graphql: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(graphql) };
    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string
    };
    CString::new("Hello ".to_owned() + recipient).unwrap().into_raw()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
