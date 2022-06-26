use crate::string::{FromSpiceString, ToSpiceString};
use crate::{Result, SPICE};
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ErrorAction {
    Abort,
    Ignore,
    Report,
    Return,
    Default,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorDevice {
    Screen,
    Null,
    Filename(String),
}

impl SPICE {
    /// Tests, retrieves, and resets the last error if it is present. Otherwise returns Ok.
    ///
    /// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/error.html#Testing%20the%20Error%20Status
    pub fn get_last_error(&self) -> Result<()> {
        unsafe {
            if failed_c() == 0 {
                return Ok(());
            }

            // Gather error info
            let mut option = "SHORT".to_spice_string();
            let mut short_message = vec![0; SPICE_ERROR_SMSGLN as usize];
            getmsg_c(
                option.as_mut_ptr(),
                short_message.len() as SpiceInt,
                short_message.as_mut_ptr(),
            );
            let mut option = "EXPLAIN".to_spice_string();
            let mut explanation = vec![0; SPICE_ERROR_XMSGLN as usize];
            getmsg_c(
                option.as_mut_ptr(),
                explanation.len() as SpiceInt,
                explanation.as_mut_ptr(),
            );
            let mut option = "LONG".to_spice_string();
            let mut long_message = vec![0; SPICE_ERROR_LMSGLN as usize];
            getmsg_c(
                option.as_mut_ptr(),
                long_message.len() as SpiceInt,
                long_message.as_mut_ptr(),
            );
            let mut traceback = vec![0; SPICE_ERROR_TRCLEN as usize];
            qcktrc_c(traceback.len() as SpiceInt, traceback.as_mut_ptr());

            // Reset last error
            reset_c();

            Err(Error {
                short_message: String::from_spice_string(&short_message),
                explanation: String::from_spice_string(&explanation),
                long_message: String::from_spice_string(&long_message),
                traceback: String::from_spice_string(&traceback),
            })
        }
    }

    /// Set the action when an error occurs in a SPICE function
    ///
    /// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/erract_c.html
    pub fn set_error_action(&self, action: ErrorAction) -> Result<()> {
        let mut set = "SET".to_spice_string();
        let mut action = serde_plain::to_string(&action).unwrap().to_spice_string();
        unsafe {
            erract_c(set.as_mut_ptr(), 0, action.as_mut_ptr());
        }
        self.get_last_error()
    }

    /// Get the action when an error occurs in a SPICE function
    ///
    /// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/erract_c.html
    pub fn get_error_action(&self) -> Result<ErrorAction> {
        let mut get = "GET".to_spice_string();
        let mut buffer = vec![0; 20];
        unsafe {
            erract_c(
                get.as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        }
        self.get_last_error()?;
        let action = String::from_spice_string(&buffer);
        Ok(serde_plain::from_str(&action).unwrap())
    }

    /// Set Error Output Device
    ///
    /// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/errdev_c.html
    pub fn set_error_output_device(&self, device: &ErrorDevice) -> Result<()> {
        let mut set = "SET".to_spice_string();
        let mut device = match &device {
            ErrorDevice::Screen => "SCREEN",
            ErrorDevice::Null => "NULL",
            ErrorDevice::Filename(filename) => filename,
        }
        .to_spice_string();
        unsafe {
            errdev_c(set.as_mut_ptr(), 0, device.as_mut_ptr());
        }
        self.get_last_error()
    }

    /// Get Error Output Device
    ///
    /// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/errdev_c.html
    pub fn get_error_output_device(&self) -> Result<ErrorDevice> {
        let mut get = "GET".to_spice_string();
        let mut buffer = vec![0; FILEN as usize];
        unsafe {
            errdev_c(
                get.as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        }
        self.get_last_error()?;
        let action = String::from_spice_string(&buffer);
        Ok(match action.as_str() {
            "SCREEN" => ErrorDevice::Screen,
            "NULL" => ErrorDevice::Null,
            filename => ErrorDevice::Filename(filename.to_owned()),
        })
    }

    pub(crate) fn set_error_defaults(&self) {
        self.set_error_action(ErrorAction::Return).unwrap();
        self.set_error_output_device(&ErrorDevice::Null).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_set_error_action() {
        let spice = SPICE::get_instance();
        spice.set_error_action(ErrorAction::Default).unwrap();
        assert_eq!(spice.get_error_action().unwrap(), ErrorAction::Default);
        spice.set_error_action(ErrorAction::Ignore).unwrap();
        assert_eq!(spice.get_error_action().unwrap(), ErrorAction::Ignore);
        spice.set_error_action(ErrorAction::Abort).unwrap();
        assert_eq!(spice.get_error_action().unwrap(), ErrorAction::Abort);

        // Reset so we don't interfere with other tests
        spice.set_error_defaults();
    }

    #[test]
    fn test_get_set_error_output_device() {
        let spice = SPICE::get_instance();
        spice.set_error_output_device(&ErrorDevice::Null).unwrap();
        assert_eq!(spice.get_error_output_device().unwrap(), ErrorDevice::Null);
        spice.set_error_output_device(&ErrorDevice::Screen).unwrap();
        assert_eq!(
            spice.get_error_output_device().unwrap(),
            ErrorDevice::Screen
        );
        let filename = ErrorDevice::Filename("errors.txt".to_owned());
        spice.set_error_output_device(&filename).unwrap();
        assert_eq!(spice.get_error_output_device().unwrap(), filename);

        // Reset so we don't interfere with other tests
        spice.set_error_defaults();
    }
}