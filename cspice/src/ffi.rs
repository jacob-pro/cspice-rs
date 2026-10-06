//! Safe wrappers over a subset of the CSPICE FFI with explicit error handling.
//!
//! The high-level API in this crate covers SPK, time and coordinates, but does not
//! expose functions such as [`pxform`], [`sxform`], [`bodvrd`], [`et2utc`] or
//! [`ktotal`]. This module wraps those entry points directly through `cspice-sys`
//! and reports every CSPICE failure as a [`SpiceFfiError`] instead of panicking
//! or terminating the process.
//!
//! # CSPICE error handling
//!
//! The CSPICE C library uses a "set failure flag" error model: a failing function
//! sets `failed_c()` to true, subsequent calls short-circuit, and `reset_c()`
//! clears the state. By default CSPICE installs the `ABORT` error action, which
//! **terminates the whole process** on the first error.
//!
//! Before the first wrapper call this module therefore switches the error action
//! to `RETURN` (the failure flag is set and the function returns) and the error
//! output device to `NULL` (messages are retrieved with `getmsg_c` and attached
//! to the returned [`SpiceFfiError`] instead of being printed to stderr).
//! Initialisation runs once via `std::sync::Once` and is idempotent.
//!
//! # Thread safety
//!
//! CSPICE global state is not thread-safe. The rest of this crate serialises all
//! calls through [`crate::with_spice_lock`]. The wrappers in this module are
//! deliberately lock-free (they are intended for tight loops where the lock
//! overhead matters); callers must either hold [`crate::with_spice_lock`] around
//! the calls or otherwise guarantee serialised access themselves.

use cspice_sys::{
    bodvrd_c, erract_c, errdev_c, et2utc_c, failed_c, getmsg_c, ktotal_c, pxform_c, qcktrc_c,
    reset_c, sxform_c, ConstSpiceChar, SpiceInt,
};
use std::ffi::CString;
use std::os::raw::c_char;
use std::sync::Once;

static ERROR_HANDLING_INIT: Once = Once::new();

/// Fetch one category of the CSPICE error message (`"SHORT"`, `"LONG"`,
/// `"EXPLAIN"`, ...; see `getmsg_c`). The buffer is truncated at the first NUL
/// byte.
fn getmsg(option: &str, len: usize) -> String {
    let opt_c = CString::new(option).expect("CSPICE option contains null byte");
    let mut msg_buf = vec![0i8; len];
    unsafe {
        getmsg_c(
            opt_c.as_ptr() as *mut ConstSpiceChar,
            len as SpiceInt,
            msg_buf.as_mut_ptr() as *mut c_char,
        );
    }
    c_chars_to_string(&msg_buf)
}

/// Fetch the CSPICE traceback (`qcktrc_c`). Truncated at the first NUL byte.
fn qcktrc(len: usize) -> String {
    let mut trace_buf = vec![0i8; len];
    unsafe {
        qcktrc_c(len as SpiceInt, trace_buf.as_mut_ptr() as *mut c_char);
    }
    c_chars_to_string(&trace_buf)
}

/// Build the full CSPICE error message: SHORT + LONG + traceback.
///
/// The caller must ensure `failed_c()` is true. SHORT is only 256 bytes and
/// frequently truncated; LONG carries the full description, and the traceback
/// points at the failing call site. The parts are joined, skipping empty ones.
fn get_full_error_message() -> String {
    let short = getmsg("SHORT", 256);
    let long = getmsg("LONG", 2048);
    let traceback = qcktrc(2048);
    let mut parts: Vec<String> = Vec::new();
    if !short.is_empty() {
        parts.push(short);
    }
    if !long.is_empty() {
        parts.push(long);
    }
    if !traceback.is_empty() {
        parts.push(format!("Traceback:\n{traceback}"));
    }
    parts.join("\n\n")
}

/// Convert a C `char` array returned by CSPICE into a `String` (truncated at the
/// first NUL byte).
fn c_chars_to_string(buf: &[i8]) -> String {
    let bytes: Vec<u8> = buf
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| c as u8)
        .collect();
    String::from_utf8_lossy(&bytes).to_string()
}

/// An error signalled by the CSPICE FFI.
#[derive(Debug)]
pub enum SpiceFfiError {
    /// `failed_c()` returned true after the call; carries the full error
    /// description (SHORT + LONG + traceback).
    Failed(String),
}

impl std::fmt::Display for SpiceFfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpiceFfiError::Failed(msg) => write!(f, "CSPICE failure: {}", msg),
        }
    }
}

impl std::error::Error for SpiceFfiError {}

/// Check the CSPICE error status, reset it and return an error if one occurred.
///
/// Call this after every CSPICE FFI call, before using its outputs.
fn check_spice_error() -> Result<(), SpiceFfiError> {
    unsafe {
        let failed: bool = failed_c() != 0;
        if failed {
            let msg = get_full_error_message();
            reset_c();
            return Err(SpiceFfiError::Failed(msg));
        }
    }
    Ok(())
}

/// Convert a Rust string into the nul-terminated C string CSPICE expects.
fn to_cstring(s: &str) -> CString {
    CString::new(s).expect("CSPICE name contains null byte")
}

/// Explicitly configure the CSPICE error action and output device.
///
/// CSPICE's default error action is `ABORT`, which terminates the process with
/// `exit(1)` on the first error. The rest of this crate installs `RETURN`/`NULL`
/// defaults the first time [`crate::with_spice_lock`] is taken; the lock-free
/// wrappers in this module may run before that ever happens. This function makes
/// the wrappers independent of the initialisation order: the action is set to
/// `RETURN` (errors set the failure flag instead of killing the process) and the
/// device to `NULL` (messages are retrieved via `getmsg_c`, not printed to
/// stderr). Runs once; repeated calls are harmless.
fn init_error_handling() {
    let set_c = CString::new("SET").unwrap();
    let return_c = CString::new("RETURN").unwrap();
    let null_c = CString::new("NULL").unwrap();
    unsafe {
        erract_c(
            set_c.as_ptr() as *mut ConstSpiceChar,
            0,
            return_c.as_ptr() as *mut ConstSpiceChar,
        );
        errdev_c(
            set_c.as_ptr() as *mut ConstSpiceChar,
            0,
            null_c.as_ptr() as *mut ConstSpiceChar,
        );
    }
}

/// Number of kernels of a given kind currently loaded in the kernel pool.
///
/// `kind` is usually `"ALL"`; other values are e.g. `"SPK"`, `"CK"`, `"EK"`,
/// `"TEXT"` and `"META"`.
///
/// See [ktotal_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/ktotal_c.html).
pub fn ktotal(kind: &str) -> Result<i32, SpiceFfiError> {
    ERROR_HANDLING_INIT.call_once(init_error_handling);
    let kind_c = to_cstring(kind);
    let mut count: SpiceInt = 0;
    unsafe {
        ktotal_c(kind_c.as_ptr() as *mut ConstSpiceChar, &mut count);
        check_spice_error()?;
    }
    Ok(count as i32)
}

/// 3x3 rotation matrix from reference frame `from` to reference frame `to` at
/// time `et` (row-major).
///
/// Equivalent to `spiceypy.pxform(from, to, et)`.
///
/// See [pxform_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/pxform_c.html).
pub fn pxform(from: &str, to: &str, et: f64) -> Result<[[f64; 3]; 3], SpiceFfiError> {
    ERROR_HANDLING_INIT.call_once(init_error_handling);
    let from_c = to_cstring(from);
    let to_c = to_cstring(to);
    let mut rotate = [[0.0_f64; 3]; 3];
    unsafe {
        pxform_c(
            from_c.as_ptr() as *mut ConstSpiceChar,
            to_c.as_ptr() as *mut ConstSpiceChar,
            et,
            rotate.as_mut_ptr(),
        );
        check_spice_error()?;
    }
    Ok(rotate)
}

/// 6x6 state transformation matrix from reference frame `from` to reference
/// frame `to` at time `et` (row-major).
///
/// Equivalent to `spiceypy.sxform(from, to, et)`.
///
/// See [sxform_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/sxform_c.html).
pub fn sxform(from: &str, to: &str, et: f64) -> Result<[[f64; 6]; 6], SpiceFfiError> {
    ERROR_HANDLING_INIT.call_once(init_error_handling);
    let from_c = to_cstring(from);
    let to_c = to_cstring(to);
    let mut xform = [[0.0_f64; 6]; 6];
    unsafe {
        sxform_c(
            from_c.as_ptr() as *mut ConstSpiceChar,
            to_c.as_ptr() as *mut ConstSpiceChar,
            et,
            xform.as_mut_ptr(),
        );
        check_spice_error()?;
    }
    Ok(xform)
}

/// Read a body attribute (e.g. `GM` or `RADII`) for a named body from the
/// kernel pool.
///
/// `maxn` is the size of the caller-provided output array and must be at least
/// the dimension of the item (e.g. 3 for `RADII`). Returns the values truncated
/// to the actual dimension, together with that dimension. Equivalent to
/// `spiceypy.bodvrd(body, item, maxn)`.
///
/// See [bodvrd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/bodvrd_c.html).
pub fn bodvrd(body: &str, item: &str, maxn: usize) -> Result<(Vec<f64>, i32), SpiceFfiError> {
    ERROR_HANDLING_INIT.call_once(init_error_handling);
    let body_c = to_cstring(body);
    let item_c = to_cstring(item);
    let mut values = vec![0.0_f64; maxn];
    let mut dim: SpiceInt = 0;
    unsafe {
        bodvrd_c(
            body_c.as_ptr() as *mut ConstSpiceChar,
            item_c.as_ptr() as *mut ConstSpiceChar,
            maxn as SpiceInt,
            &mut dim,
            values.as_mut_ptr(),
        );
        check_spice_error()?;
    }
    values.truncate(dim as usize);
    Ok((values, dim))
}

/// Convert ephemeris time (ET, seconds past J2000 TDB) to a UTC calendar string
/// in `"ISOC"` format with `prec` fractional-second digits, e.g.
/// `"2000-01-01T11:58:55.816"`.
///
/// Equivalent to `spiceypy.et2utc(et, "ISOC", prec)`.
///
/// See [et2utc_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/et2utc_c.html).
pub fn et2utc(et: f64, prec: i32) -> Result<String, SpiceFfiError> {
    ERROR_HANDLING_INIT.call_once(init_error_handling);
    let fmt_c = to_cstring("ISOC");
    let mut buf = vec![0i8; 64];
    unsafe {
        et2utc_c(
            et,
            fmt_c.as_ptr() as *mut ConstSpiceChar,
            prec as SpiceInt,
            buf.len() as SpiceInt,
            buf.as_mut_ptr() as *mut c_char,
        );
        check_spice_error()?;
    }
    Ok(c_chars_to_string(&buf))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::unload;
    use crate::tests::load_test_data;
    use crate::with_spice_lock;

    /// Serialise SPICE access across the whole test binary: the wrappers
    /// themselves are lock-free, so the tests hold the library lock explicitly.
    fn serialize<R>(f: impl FnOnce() -> R) -> R {
        with_spice_lock(f)
    }

    #[test]
    fn ktotal_counts_loaded_kernels() {
        let (all, spk) = serialize(|| {
            load_test_data();
            (ktotal("ALL").unwrap(), ktotal("SPK").unwrap())
        });
        // The meta-kernel registers as META, naif0012.tls as TEXT, de432s.bsp
        // as SPK. There is no "LSK" kind in ktotal.
        assert_eq!(spk, 1);
        assert_eq!(ktotal("TEXT").unwrap(), 1);
        assert_eq!(ktotal("META").unwrap(), 1);
        assert_eq!(all, 3);
    }

    #[test]
    fn pxform_returns_orthonormal_rotation() {
        let r = serialize(|| {
            load_test_data();
            pxform("J2000", "ECLIPJ2000", 0.0)
        })
        .unwrap();
        for row in r.iter() {
            let norm = (row[0] * row[0] + row[1] * row[1] + row[2] * row[2]).sqrt();
            assert!((norm - 1.0).abs() < 1e-10, "row norm {norm} != 1");
        }
    }

    #[test]
    fn sxform_state_matrix_has_orthonormal_rotation_blocks() {
        let x = serialize(|| {
            load_test_data();
            sxform("J2000", "ECLIPJ2000", 0.0)
        })
        .unwrap();
        // For a constant inertial rotation the state matrix is [[R, 0], [0, R]]
        // with dR/dt = 0: the two diagonal quadrants are the same orthonormal
        // rotation and the off-diagonal quadrants are zero.
        let top = [
            [x[0][0], x[0][1], x[0][2]],
            [x[1][0], x[1][1], x[1][2]],
            [x[2][0], x[2][1], x[2][2]],
        ];
        let bottom = [
            [x[3][3], x[3][4], x[3][5]],
            [x[4][3], x[4][4], x[4][5]],
            [x[5][3], x[5][4], x[5][5]],
        ];
        assert_eq!(top, bottom);
        for row in top.iter() {
            let norm = (row[0] * row[0] + row[1] * row[1] + row[2] * row[2]).sqrt();
            assert!((norm - 1.0).abs() < 1e-10, "row norm {norm} != 1");
        }
        for (i, row) in x.iter().enumerate() {
            for (j, &value) in row.iter().enumerate() {
                if (i < 3) != (j < 3) {
                    assert!(value == 0.0, "x[{i}][{j}] = {value} != 0");
                }
            }
        }
    }

    #[test]
    fn bodvrd_reads_radii_from_pck() {
        serialize(|| {
            load_test_data();
            // de432s.bsp + naif0012.tls contain no PCK data, so furnish a small
            // text PCK with MARS radii.
            let pck = std::env::temp_dir()
                .join(format!("cspice_ffi_test_pck_{}.tps", std::process::id()));
            std::fs::write(
                &pck,
                "\\begindata\nBODY499_RADII = ( 3396.19  3396.19  3376.2 )\n\\begintext\n",
            )
            .unwrap();
            crate::data::furnish(pck.to_string_lossy().to_string()).unwrap();
            let (values, dim) = bodvrd("MARS", "RADII", 3).unwrap();
            assert_eq!(dim, 3);
            assert_eq!(values, vec![3396.19, 3396.19, 3376.2]);
            unload(pck.to_string_lossy().to_string()).unwrap();
            let _ = std::fs::remove_file(&pck);
        });
    }

    #[test]
    fn et2utc_known_epochs() {
        serialize(|| {
            load_test_data();
            // ET = 0 is the J2000 epoch (2000-01-01T12:00:00 TDB), which is
            // 11:58:55.816 UTC (32.184 s TAI-UTC + 27 s leap seconds).
            assert_eq!(et2utc(0.0, 3).unwrap(), "2000-01-01T11:58:55.816");
            // 2000-01-01T00:00:00 UTC.
            assert_eq!(
                et2utc(-43135.816079952438, 3).unwrap(),
                "2000-01-01T00:00:00.000"
            );
        });
    }

    #[test]
    fn errors_are_reported_not_fatal() {
        // Without the RETURN error action installed by this module, CSPICE's
        // default ABORT action would terminate the test process here instead
        // of returning an error.
        let error = serialize(|| {
            load_test_data();
            pxform("NOT_A_REAL_FRAME", "J2000", 0.0)
        })
        .unwrap_err();
        assert!(error.to_string().contains("SPICE"), "{error}");

        let error = bodvrd("BODYX9", "RADII", 3).unwrap_err();
        assert!(error.to_string().contains("SPICE"), "{error}");
    }
}
