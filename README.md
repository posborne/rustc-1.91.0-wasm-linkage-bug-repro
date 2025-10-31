# Rust WASM32-WASIP1 Symbol Mangling Bug Reproduction

This repository contains minimal reproductions for a bug introduced in Rust
1.91.0 that affects building for the `wasm32-wasip1` target.

## Bug Description

The issue occurs when building Rust code for `wasm32-wasip1` (and probably
similar targets) that uses WASM import modules. The bug manifests as a linker
error due to import module mismatch for symbols:

```
rust-lld: error: import module mismatch for symbol: close
>>> defined as fastly_http_req in fastly_repro-ded4f062b7633b63.fastly_repro.83241ecddd8cca01-cgu.0.rcgu.o
>>> defined as env in libstd-e45a67150aa246d3.rlib
```

## Root Cause Analysis

We first encountered this issue with the 1.91.0 upgrade, but bisecting shows
that this bug was introduced by commit
[8e62bfd311791bfd9dca886abdfbab07ec54d8b4](https://github.com/rust-lang/rust/commit/8e62bfd311791bfd9dca886abdfbab07ec54d8b4)
and was not caught in the wild during the nightly/beta periods..

**The Problem:** The commit changed how foreign items with WASM import modules
are handled, introducing overly aggressive mangling that conflicts with stdlib
symbols.

This writeup contains my best guess as to what is going wrong.

**What Changed:**

1. **Foreign items now auto-get `no_mangle`** - All foreign items automatically
   receive the `NO_MANGLE` flag
2. **WASM exception forces mangling** - A new exception forces mangling for WASM
   import modules, overriding both `#[link_name]` and `no_mangle`
3. **Logic precedence changed** - The order of `link_name` vs `no_mangle` vs
   mangling logic was reorganized

**The Bug:**

1. Our `close` symbol has `#[link_name = "close"]` and gets auto-`no_mangle`
2. WASM exception (`wasm_import_module_exception_force_mangling`) kicks in
   because it has `wasm_import_module = "test"`
3. Both `link_name` and `no_mangle` are bypassed, forcing symbol mangling
4. Our symbol becomes mangled (e.g., `_ZN...close...`) instead of literal
   "close"
5. Stdlib's `close` symbol remains unmangled as "close"
6. Linker sees conflict between mangled vs unmangled versions of "close"

**Complicating Factors:**

Unfortuantely, when LTO is enabled the problem is more severe as there is no
linker error and, instead, linking succeeds and the module will encounter
runtime errors. In the fastly case, this exhibited as a wasm trap due to
validation failures at the ABI boundary for the errantly linked hostcalls.

This behavior with LTO seems similar to
[this LLVM issue](https://bugs.llvm.org/show_bug.cgi?id=44316) which was
resolved some time ago, at least for the non-LTO case. I haven't dug into this
side of things in detail.

## Reproductions

This workspace contains two reproductions:

### 1. Fastly-sys Reproduction (`fastly-repro/`)

A minimal reproduction using actual `fastly-sys` and `fastly-shared` crates that
demonstrates the real-world impact of this bug. This is where the rustc issue
was first encountered in the wild, shortly after the 1.91.0 release.

**To reproduce:**

```bash
cargo build --target wasm32-wasip1 --release -p fastly-repro
```

**Expected behavior:** Should build successfully

**Actual behavior:** Fails with import module mismatch error

### 2. Standalone Reproduction (`standalone-repro/`)

A self-contained reproduction that recreates the exact symbol conflict structure
without requiring external dependencies. This demonstrates the core issue with
WASM import module symbol handling using a multi-crate setup.

**To reproduce:**

```bash
cargo build --target wasm32-wasip1 --release -p standalone-repro
```

**Expected behavior:** Should build successfully

**Actual behavior:** Fails with the same import module mismatch error as
fastly-repro

**Key insights:**

- The bug requires **multiple crates/rlibs** to trigger the linking scenario
  where symbol conflicts occur between separate object files
- The standalone reproduction includes a separate `mock-wasm-imports` crate that
  creates a minimal WASM import module symbol that conflicts with stdlib
- **CGU splitting:** The fastly-repro fails with default settings, but the
  standalone-repro requires `codegen-units = 16` to force multiple codegen units
  that trigger the conflict

## Environment

- **Affected Rust Version:** 1.91.0+
- **Target:** wasm32-wasip1
- **Profile Settings:**
  - `codegen-units = 16` (required for standalone-repro, optional for
    fastly-repro)
  - `lto = false`
  - `panic = "abort"`

## Impact

This bug affects any project that:

1. Targets `wasm32-wasip1`
2. Uses crates with WASM import modules (like `fastly-sys`) that conflict with
   libc or other symbols coming from outside of rust.
3. Project has complexity to require linkage of multiple CGUs.

Notably, this impacts the entire Fastly Compute ecosystem and potentially other
WASM-based projects.
