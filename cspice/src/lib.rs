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

use crate::error::set_error_defaults;
pub use crate::error::Error;
use crate::string::SpiceString;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use thiserror::Error;

// Boolean indicates if library has been initialised
static SPICE_LOCK: ReentrantMutex<RefCell<bool>> = ReentrantMutex::new(RefCell::new(false));

pub(crate) fn with_spice_lock_or_panic<R, F>(f: F) -> R
where
    F: FnOnce() -> R,
{
    match try_with_spice_lock(f) {
        Ok(k) => k,
        Err(e) => {
            panic!("{e}")
        }
    }
}

/// The SPICE library [is not thread safe](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/problems.html#Problem:%20SPICE%20code%20is%20not%20thread%20safe.).
/// This function can be used to synchronise calls to SPICE functions.
/// All safe functions in this library use this lock internally.
pub fn try_with_spice_lock<R, F>(f: F) -> Result<R, SpiceLockError>
where
    F: FnOnce() -> R,
{
    let guard = SPICE_LOCK.try_lock().ok_or(SpiceLockError)?;
    initialise_library(&guard);
    Ok(f())
}

/// The SPICE library [is not thread safe](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/problems.html#Problem:%20SPICE%20code%20is%20not%20thread%20safe.).
/// This function can be used to synchronise calls to SPICE functions.
/// All safe functions in this library use this lock internally.
/// The lock is reentrant.
pub fn with_spice_lock<R, F>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let guard = SPICE_LOCK.lock();
    initialise_library(&guard);
    f()
}

fn initialise_library(guard: &ReentrantMutexGuard<'static, RefCell<bool>>) {
    if !guard.borrow().deref() {
        *guard.borrow_mut() = true;
        set_error_defaults();
    }
}

#[derive(Debug)]
pub struct SpiceLock(ReentrantMutexGuard<'static, RefCell<bool>>);

/// Error returned from [try_with_spice_lock()].
#[derive(Debug, Clone, Error)]
#[cfg_attr(not(test), error("SPICE is already in use by another thread. If multi-threaded use is intentional wrap the call using `with_spice_lock()`."))]
#[cfg_attr(test, error("SPICE is already in use by another thread. When running unit tests you will likely need to use the `--test-threads=1` argument."))]
pub struct SpiceLockError;

#[cfg(test)]
mod tests {
    use crate::data::furnish;
    use std::path::PathBuf;
    use std::sync::Once;

    /// Load test data (once)
    pub fn load_test_data() {
        static SPICE_INIT: Once = Once::new();
        SPICE_INIT.call_once(|| {
            let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data");
            furnish(data_dir.join("testkernel.txt").to_string_lossy()).unwrap();
        });
    }
}
