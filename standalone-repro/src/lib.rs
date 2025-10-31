// Test if multi-crate rlib linking triggers the stdlib conflict

use std::fs::File;
use std::os::fd::FromRawFd;
use mock_wasm_imports::close;

#[no_mangle]
pub extern "C" fn use_custom_close() {
    unsafe { close(1); }
}

#[no_mangle]
pub extern "C" fn use_stdlib_close() {
    unsafe { let _file = File::from_raw_fd(999); }
}

#[export_name = "_start"]
pub extern "C" fn main() {
    use_custom_close();
    use_stdlib_close();
}