pub mod cell;
pub mod common;
pub mod data;
pub mod error;
pub mod string;
pub mod time;

use crate::error::Error;
use crate::string::SpiceString;
use once_cell::sync::Lazy;
use std::fmt::{Debug, Formatter};
use std::ptr::null_mut;
use std::sync::Mutex;
use std::thread;
use std::thread::ThreadId;
use thiserror::Error;

/// The `Spice` struct is used to ensure that the SPICE functions are only ever called from the same
/// thread.
///
/// The SPICE library [is not thread safe](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/problems.html#Problem:%20SPICE%20code%20is%20not%20thread%20safe).
///
/// # Example
///
/// ```compile_fail
/// # use cspice::SPICE;
/// // You can't share this across threads - it will not compile:
/// let spice = SPICE::get_instance();
/// std::thread::spawn(|| spice );
/// ```
#[derive(Copy, Clone)]
// Note: a pointer is used to make this !Send, until feature negative_impls is stabilised
pub struct Spice(*mut u8);

impl Debug for Spice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("SPICE")
    }
}

#[derive(Debug, Clone, Error)]
#[error("SPICE is already in use by another thread")]
pub struct SpiceThreadError;

impl Spice {
    /// Get an instance of the `Spice` struct, allowing you to call SPICE functions.
    ///
    /// Note: The first call to this function will set SPICE error action to RETURN, errors will
    /// be handled using Rust's Result type.
    ///
    /// SPICE will be locked to the first thread that calls this function.
    pub fn try_get_instance() -> Result<Spice, SpiceThreadError> {
        static SPICE_THREAD_ID: Lazy<Mutex<Option<ThreadId>>> = Lazy::new(|| Mutex::new(None));

        let mut thread_id = SPICE_THREAD_ID.lock().unwrap();
        match &*thread_id {
            None => {
                *thread_id = Some(thread::current().id());
                let spice = Spice(null_mut());
                spice.set_error_defaults();
                Ok(spice)
            }
            Some(thread_id) => {
                if *thread_id == thread::current().id() {
                    return Ok(Spice(null_mut()));
                }
                Err(SpiceThreadError)
            }
        }
    }

    /// Get an instance of the `Spice` struct, allowing you to call SPICE functions.
    ///
    /// Note: The first call to this function will set SPICE error action to RETURN, errors will
    /// be handled using Rust's Result type.
    ///
    /// # Panics
    ///
    /// If you call this from a second thread, it will panic.
    pub fn get_instance() -> Spice {
        Self::try_get_instance().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::Spice;
    use std::path::PathBuf;
    use std::sync::Once;

    /// A SPICE instance with test data loaded, for use in unit tests
    pub fn get_test_spice() -> Spice {
        static SPICE_INIT: Once = Once::new();
        let spice = Spice::get_instance();
        SPICE_INIT.call_once(|| {
            let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
            spice
                .furnish(data_dir.join("naif0012.tls").to_string_lossy())
                .unwrap();
        });
        return spice;
    }

    #[test]
    fn test_get_instance() {
        let _one = Spice::get_instance();
        let _two = Spice::get_instance();
    }

    #[test]
    fn test_get_instance_different_thread() {
        let _first = Spice::get_instance();
        std::thread::spawn(|| {
            Spice::try_get_instance().expect_err("Should be unable to use on another thread")
        })
        .join()
        .unwrap();
    }
}
