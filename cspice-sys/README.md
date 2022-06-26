# cspice-sys

[![Build](https://github.com/jacob-pro/cspice-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/jacob-pro/cspice-rs/actions)

Unsafe bindings to the NAIF [SPICE Toolkit](https://naif.jpl.nasa.gov/naif/index.html).

Read the [official CSPICE documentation online](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/index.html)

## Installation

You must first have installed the CSPICE toolkit by downloading and extracting the appropriate archive from 
[here](https://naif.jpl.nasa.gov/naif/toolkit_C.html).

You must set the `CSPICE_DIR` environment variable to point to the extracted `cspice` directory (which should contain
the `include` and `lib` directories).

**WARNING**: On Unix like systems you will likely need to rename `lib/cspice.a` to `lib/libcspice.a` so that it can be
successfully linked.

You must also have [Clang](https://releases.llvm.org/download.html) installed and on your `PATH` to be able to generate 
the bindings. 
