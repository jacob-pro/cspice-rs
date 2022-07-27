pub mod cell;
pub mod common;
pub mod coordinates;
pub mod data;
pub mod error;
pub mod gf;
pub mod spk;
pub mod string;
pub mod time;
pub mod vector;

pub use crate::error::Error;

use crate::error::set_error_defaults;
use crate::string::SpiceString;
use once_cell::sync::OnceCell;
use std::cell::Cell;
use std::fmt::Debug;
use std::thread;
use std::thread::Thread;
use thiserror::Error;

/// Wraps an unsafe SPICE function call.
///
/// First checks that it is safe for the current thread to access SPICE, otherwise panics.
macro_rules! spice_unsafe {
    ($l:block) => {{
        if let Err(e) = crate::try_acquire_thread() {
            panic!("{e}")
        }
        unsafe { $l }
    }};
}
pub(crate) use spice_unsafe;

/// The SPICE library [is not thread safe](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/problems.html#Problem:%20SPICE%20code%20is%20not%20thread%20safe).
/// This function checks if it is safe for the current thread to call SPICE functions. SPICE
/// will be locked to the first thread that calls this function.
pub fn try_acquire_thread() -> Result<(), SpiceThreadError> {
    static SPICE_THREAD_ID: OnceCell<Thread> = OnceCell::new();
    thread_local! {
        static CACHE: Cell<bool> = Cell::new(false);
    }
    CACHE.with(|cache| {
        if cache.get() {
            return Ok(());
        }
        match SPICE_THREAD_ID.set(thread::current()) {
            Ok(_) => {
                cache.set(true);
                set_error_defaults();
                Ok(())
            }
            Err(e) => Err(SpiceThreadError(e)),
        }
    })
}

/// Error returned from [try_acquire_thread()].
#[derive(Debug, Clone, Error)]
#[cfg_attr(not(test), error("SPICE is already in use by another thread"))]
#[cfg_attr(test, error("SPICE is already in use by another thread. When running unit tests you will likely need to use the `--test-threads=1` argument"))]
pub struct SpiceThreadError(pub Thread);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::furnish;
    use std::path::PathBuf;
    use std::sync::Once;

    /// Load test data (once)
    pub fn load_test_data() {
        static SPICE_INIT: Once = Once::new();
        SPICE_INIT.call_once(|| {
            let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
            furnish(data_dir.join("naif0012.tls").to_string_lossy()).unwrap();
        });
    }

    #[test]
    fn test_acquire_thread() {
        try_acquire_thread().unwrap();
        try_acquire_thread().unwrap();
    }

    #[test]
    fn test_acquire_thread_different_thread() {
        try_acquire_thread().unwrap();
        std::thread::spawn(|| {
            try_acquire_thread().expect_err("Should be unable to use on another thread")
        })
        .join()
        .unwrap();
    }
}
