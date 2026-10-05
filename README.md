# cspice-rs

Rust bindings for [CSPICE](https://naif.jpl.nasa.gov/naif/toolkit.html), the spacecraft geometry
toolkit from NASA/JPL's
[Navigation and Ancillary Information Facility](https://naif.jpl.nasa.gov/). SPICE provides
ephemerides, reference frame transformations, time system conversions and orientation data for
planetary science and mission design.

| Crate | Description |
| --- | --- |
| [`cspice-sys`](./cspice-sys) | Raw unsafe FFI bindings to the full CSPICE API, generated with [bindgen](https://github.com/rust-lang/rust-bindgen). |
| [`cspice`](./cspice) | Safe, thread-serialised wrappers around the most commonly used subsystems. |

The safe API is still growing. For anything not yet wrapped, `cspice-sys` exposes the complete
CSPICE C API as `unsafe` functions.

## Building

`cspice-sys` links against the CSPICE C library, which is **not bundled** with this repository.
There are three ways to provide it:

1. **Download during build** (easiest): enable the `downloadcspice` feature of `cspice-sys`, which
   fetches a prebuilt package from
   [naif.jpl.nasa.gov](https://naif.jpl.nasa.gov/pub/naif/toolkit/C/) into the build directory.
   Supported for 64-bit Linux, macOS and Windows.
2. **Environment variable**: set `CSPICE_DIR` to the path of an unpacked
   [CSPICE toolkit](https://naif.jpl.nasa.gov/naif/toolkit.html) containing `include/SpiceUsr.h`
   and `lib/libcspice.a`.
3. **System installation**: if `/usr/lib/libcspice.a` exists on Linux or macOS it is used
   automatically.

Bindings are generated at build time, so [libclang](https://clang.llvm.org/) must be available.
For cross-compilation the bindgen target and sysroot can be overridden with the `CSPICE_CLANG_TARGET`
and `CSPICE_CLANG_ROOT` environment variables.

```toml
[dependencies]
cspice = "0.1"

# Option 1 only: download CSPICE from NAIF during the cspice-sys build
cspice-sys = { version = "1.0", features = ["downloadcspice"] }
```

## Usage

```rust
use cspice::common::AberrationCorrection;
use cspice::data::furnish;
use cspice::spk::easier_reader;
use cspice::time::Et;

fn main() -> Result<(), cspice::Error> {
    // Load kernels, e.g. from https://naif.jpl.nasa.gov/pub/naif/generic_kernels/
    furnish("naif0012.tls")?; // leap seconds kernel, required for time conversion
    furnish("de432s.bsp")?;   // planetary ephemeris

    // Parse a UTC string into ephemeris time, seconds past J2000 TDB
    let et = Et::from_string("2026-10-05T12:00:00")?;

    // State of the Moon relative to Earth in the J2000 frame, corrected for light time
    let (state, light_time) = easier_reader("MOON", et, "J2000", AberrationCorrection::LT, "EARTH")?;

    println!(
        "Moon wrt Earth: x = {} km, y = {} km, z = {} km, one-way light time = {} s",
        state.position.x, state.position.y, state.position.z, light_time
    );
    Ok(())
}
```

## Error handling

CSPICE's default error action is `ABORT`, which terminates the whole process. This crate switches
the action to `RETURN` and the error device to `NULL` before first use, so every wrapper returns
a `Result` instead: failures are reported as [`cspice::Error`](./cspice/src/error.rs) values
carrying the short message, explanation, long message and traceback from the CSPICE error
machinery. The action and device can be changed at runtime through the `cspice::error` module.

## Thread safety

SPICE keeps its state in globals and
[is not thread safe](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/problems.html#Problem:%20SPICE%20code%20is%20not%20thread%20safe.).
All safe functions in this crate therefore serialise their calls through an internal reentrant
mutex. If you call into `cspice-sys` directly, wrap those calls with [`cspice::with_spice_lock()`]
or [`cspice::try_with_spice_lock()`] to keep the rest of the library sound.

## Coverage

The `cspice` crate currently wraps:

- **Kernel management** — `furnsh` / `unload`
- **Ephemeris (SPK)** — position and state queries by body name or NAIF ID, with aberration
  corrections
- **Time** — `Et` seconds-past-J2000, string conversion in both directions, calendars, Julian
  dates, `DateTime` and `JulianDate` types
- **Coordinates** — rectangular, azimuth/elevation and related systems
- **Error handling** — error action and output device configuration, structured errors
- **Cells and windows** — SPICE `SpiceCell` based interval arithmetic
- **Geometry finder** — angular separation searches

## License

This repository is licensed under the GNU LGPL-3.0. CSPICE itself is distributed by NAIF under
its [own terms](https://naif.jpl.nasa.gov/naif/toolkit.html) and is only downloaded or linked at
build time.
