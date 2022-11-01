# cspice-sys

[![Build](https://github.com/jacob-pro/cspice-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jacob-pro/cspice-rs/actions)
[![crates.io](https://img.shields.io/crates/v/cspice-sys.svg)](https://crates.io/crates/cspice-sys)
[![docs.rs](https://docs.rs/cspice-sys/badge.svg)](https://docs.rs/cspice-sys/latest/cspice_sys/)

Unsafe bindings to the NAIF [SPICE Toolkit](https://naif.jpl.nasa.gov/naif/index.html).

Read the [official CSPICE documentation online](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/index.html)

*Please note this is a complete replacement of the 0.x version of the crate, under a new maintainer*

## Installation

Firstly, you must have [Clang](https://releases.llvm.org/download.html) installed and on your `PATH` to be able to generate 
the bindings.

If you're on a Unix-like system and have CSPICE installed in the standard paths (`libcspice.a` in `/usr/lib` and headers in `/usr/include`),
that version will be used by default.

Alternatively, you can enable the `downloadcspice` feature on the crate to automatically download CSPICE from NAIF servers
when this crate is built. Be aware that this will increase build time and require an internet connection on every clean build.

You can also download CSPICE and tell this crate about where to find it manually:
First install the CSPICE toolkit by downloading and extracting the appropriate archive from 
[here](https://naif.jpl.nasa.gov/naif/toolkit_C.html).

Then, set the `CSPICE_DIR` environment variable to point to the extracted `cspice` directory (which should contain
the `include` and `lib` directories).

**WARNING**: On Unix like systems you will likely need to rename `lib/cspice.a` to `lib/libcspice.a` so that it can be
successfully linked.

Also see the [GitHub workflow](../.github/workflows/rust.yml) for examples on how to set this up.

## Cross Compilation

You can use the `CSPICE_CLANG_TARGET` environment variable to override the `--target` parameter for Clang (when 
generating bindings).

You can use the `CSPICE_CLANG_ROOT` environment variable to override the `--sysroot` parameter for Clang (when 
generating bindings).
