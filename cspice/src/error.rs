use crate::common::{GET, SET};
use crate::string::{SpiceStr, SpiceString};
use crate::{spice_unsafe, Spice};
use cspice_sys::{
    erract_c, errdev_c, failed_c, getmsg_c, qcktrc_c, reset_c, SpiceInt, SPICE_ERROR_LMSGLN,
    SPICE_ERROR_SMSGLN, SPICE_ERROR_TRCLEN, SPICE_ERROR_XMSGLN,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const FILEN: SpiceInt = 255;

#[derive(Debug, Clone, Error)]
#[error("{short_message}\n\n{explanation}\n\n{long_message}\n\nTraceback:\n{traceback}")]
pub struct Error {
    pub short_message: String,
    pub explanation: String,
    pub long_message: String,
    pub traceback: String,
}

/// See [Choosing the Error Response Action](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/error.html#Choosing%20the%20Error%20Response%20Action).
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ErrorAction {
    Abort,
    Ignore,
    Report,
    Return,
    Default,
}

/// See [Choosing Where the Error Messages Are Sent](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/error.html#Choosing%20Where%20the%20Error%20Messages%20Are%20Sent).
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorDevice {
    Screen,
    Null,
    Filename(String),
}

/// Functions relating to error handling.
impl Spice {
    /// Tests, retrieves, and resets the last error if it is present. Otherwise returns Ok.
    ///
    /// For context see [CSPICE Error Handling](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/error.html#Testing%20the%20Error%20Status).
    #[inline]
    pub fn get_last_error() -> Result<(), Error> {
        spice_unsafe!({
            if failed_c() == 0 {
                return Ok(());
            }

            // Gather error info
            let option = SpiceString::from("SHORT");
            let mut short_message = [0; SPICE_ERROR_SMSGLN as usize];
            getmsg_c(
                option.as_mut_ptr(),
                short_message.len() as SpiceInt,
                short_message.as_mut_ptr(),
            );
            let option = SpiceString::from("EXPLAIN");
            let mut explanation = [0; SPICE_ERROR_XMSGLN as usize];
            getmsg_c(
                option.as_mut_ptr(),
                explanation.len() as SpiceInt,
                explanation.as_mut_ptr(),
            );
            let option = SpiceString::from("LONG");
            let mut long_message = [0; SPICE_ERROR_LMSGLN as usize];
            getmsg_c(
                option.as_mut_ptr(),
                long_message.len() as SpiceInt,
                long_message.as_mut_ptr(),
            );
            let mut traceback = [0; SPICE_ERROR_TRCLEN as usize];
            qcktrc_c(traceback.len() as SpiceInt, traceback.as_mut_ptr());

            // Reset last error
            reset_c();

            Err(Error {
                short_message: SpiceStr::from_buffer(&short_message).to_string(),
                explanation: SpiceStr::from_buffer(&explanation).to_string(),
                long_message: SpiceStr::from_buffer(&long_message).to_string(),
                traceback: SpiceStr::from_buffer(&traceback).to_string(),
            })
        })
    }

    /// Set the action when an error occurs in a SPICE function.
    ///
    /// See [erract_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/erract_c.html).
    pub fn set_error_action(action: ErrorAction) -> Result<(), Error> {
        let action = SpiceString::from(serde_plain::to_string(&action).unwrap());
        spice_unsafe!({
            erract_c(SET.as_mut_ptr(), 0, action.as_mut_ptr());
        });
        Spice::get_last_error()
    }

    /// Get the action when an error occurs in a SPICE function.
    ///
    /// See [erract_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/erract_c.html).
    pub fn get_error_action() -> Result<ErrorAction, Error> {
        let mut buffer = [0; 20];
        spice_unsafe!({
            erract_c(
                GET.as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        });
        Self::get_last_error()?;
        let action = SpiceStr::from_buffer(&buffer);
        Ok(serde_plain::from_str(&*action.as_str()).unwrap())
    }

    /// Set Error Output Device.
    ///
    /// See [errdev_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/errdev_c.html).
    pub fn set_error_output_device(device: ErrorDevice) -> Result<(), Error> {
        let device = match device {
            ErrorDevice::Screen => SpiceString::from("SCREEN"),
            ErrorDevice::Null => SpiceString::from("NULL"),
            ErrorDevice::Filename(filename) => SpiceString::from(filename),
        };
        spice_unsafe!({
            errdev_c(SET.as_mut_ptr(), 0, device.as_mut_ptr());
        });
        Self::get_last_error()
    }

    /// Get Error Output Device.
    ///
    /// See [errdev_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/errdev_c.html).
    pub fn get_error_output_device() -> Result<ErrorDevice, Error> {
        let mut buffer = [0; FILEN as usize];
        spice_unsafe!({
            errdev_c(
                GET.as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        });
        Self::get_last_error()?;
        let action = SpiceStr::from_buffer(&buffer);
        Ok(match action.as_str() {
            s if s == "SCREEN" => ErrorDevice::Screen,
            s if s == "NULL" => ErrorDevice::Null,
            s => ErrorDevice::Filename(s.into_owned()),
        })
    }

    pub(crate) fn set_error_defaults() {
        Self::set_error_action(ErrorAction::Return).unwrap();
        Self::set_error_output_device(ErrorDevice::Null).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set_error_action() {
        Spice::set_error_action(ErrorAction::Default).unwrap();
        assert_eq!(Spice::get_error_action().unwrap(), ErrorAction::Default);
        Spice::set_error_action(ErrorAction::Ignore).unwrap();
        assert_eq!(Spice::get_error_action().unwrap(), ErrorAction::Ignore);
        Spice::set_error_action(ErrorAction::Abort).unwrap();
        assert_eq!(Spice::get_error_action().unwrap(), ErrorAction::Abort);

        // Reset so we don't interfere with other tests
        Spice::set_error_defaults();
    }

    #[test]
    fn test_get_set_error_output_device() {
        Spice::set_error_output_device(ErrorDevice::Null).unwrap();
        assert_eq!(Spice::get_error_output_device().unwrap(), ErrorDevice::Null);
        Spice::set_error_output_device(ErrorDevice::Screen).unwrap();
        assert_eq!(
            Spice::get_error_output_device().unwrap(),
            ErrorDevice::Screen
        );
        let filename = ErrorDevice::Filename(String::from("errors.txt"));
        Spice::set_error_output_device(filename.clone()).unwrap();
        assert_eq!(Spice::get_error_output_device().unwrap(), filename);

        // Reset so we don't interfere with other tests
        Spice::set_error_defaults();
    }
}
