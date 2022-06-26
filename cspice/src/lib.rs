mod data;
pub mod error;
mod string;

use crate::error::Error;
use once_cell::sync::Lazy;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::thread;
use std::thread::ThreadId;

pub type Result<T> = std::result::Result<T, Error>;

/// SPICE is not a thread safe library.
///
/// https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/problems.html#Problem:%20SPICE%20code%20is%20not%20thread%20safe.
///
/// The SPICE struct is used to ensure that the SPICE functions are only ever called from the same
/// thread.
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
pub struct SPICE(*mut u8);

impl SPICE {
    /// Get an instance of the SPICE struct, allowing you to call SPICE functions.
    ///
    /// Note: The first call to this function will set SPICE error action to RETURN, errors will
    /// be handled using Rust's Result type.
    ///
    /// SPICE will be locked to the first thread that calls this function.
    ///
    /// # Panics
    ///
    /// If you call this from a second thread, it will panic.
    pub fn get_instance() -> SPICE {
        static SPICE_THREAD_ID: Lazy<Mutex<Option<ThreadId>>> = Lazy::new(|| Mutex::new(None));

        let mut thread_id = SPICE_THREAD_ID.lock().unwrap();
        match &*thread_id {
            None => {
                *thread_id = Some(thread::current().id());
                let spice = SPICE(null_mut());
                spice.set_error_defaults();
                spice
            }
            Some(thread_id) => {
                if *thread_id == thread::current().id() {
                    return SPICE(null_mut());
                }
                panic!("SPICE is in use by another thread")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SPICE;

    #[test]
    fn test_get_instance() {
        let _one = SPICE::get_instance();
        let _two = SPICE::get_instance();
    }

    #[test]
    #[should_panic]
    fn test_get_instance_different_thread() {
        std::thread::spawn(|| {
            let _first = SPICE::get_instance();
        })
        .join()
        .unwrap();
        let _second = SPICE::get_instance();
    }
}
