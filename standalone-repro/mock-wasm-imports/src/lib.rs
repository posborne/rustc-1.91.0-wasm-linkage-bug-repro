#![no_std]

// Mock library with close symbol that conflicts with stdlib

#[link(wasm_import_module = "test")]
extern "C" {
    #[link_name = "close"]
    pub fn close(x: u32) -> u32;
}