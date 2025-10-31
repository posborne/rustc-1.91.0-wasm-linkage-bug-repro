#![no_std]

// Second mock library with the SAME symbol name but different import module

#[link(wasm_import_module = "module_b")]
extern "C" {
    #[link_name = "foo"]  // Same symbol name as mock-wasm-imports
    pub fn foo(x: u32) -> u32;
}