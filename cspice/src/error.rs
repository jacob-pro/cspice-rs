use crate::string::{FromSpiceString, ToSpiceString};
use crate::{Result, SPICE};
use cspice_sys::{
    erract_c, failed_c, getmsg_c, qcktrc_c, reset_c, SpiceInt, SPICE_ERROR_LMSGLN,
    SPICE_ERROR_SMSGLN, SPICE_ERROR_TRCLEN, SPICE_ERROR_XMSGLN,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{short_message}\n\n{explanation}\n\n{long_message}\n\nTraceback:\n{traceback}")]
pub struct Error {
    pub short_message: String,
    pub explanation: String,
    pub long_message: String,
    pub traceback: String,
}

#[derive(Debug, Copy, Clone, Serialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ErrorAction {
    Abort,
    Ignore,
    Report,
    Return,
    Default,
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
            erract_c(get.as_mut_ptr(), get.len() as SpiceInt, buffer.as_mut_ptr());
        }
        self.get_last_error()?;
        let action = String::from_spice_string(&buffer);
        Ok(match action.as_str() {
            "ABO" => ErrorAction::Abort,
            "IGN" => ErrorAction::Ignore,
            "REP" => ErrorAction::Report,
            "RET" => ErrorAction::Return,
            "DEF" => ErrorAction::Default,
            _ => panic!("Unknown error action: {}", action),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{ErrorAction, SPICE};

    #[test]
    fn test_get_set_error_action() {
        let spice = SPICE::get_instance();
        spice.set_error_action(ErrorAction::Default).unwrap();
        assert_eq!(spice.get_error_action().unwrap(), ErrorAction::Default);
        spice.set_error_action(ErrorAction::Ignore).unwrap();
        assert_eq!(spice.get_error_action().unwrap(), ErrorAction::Ignore);
        spice.set_error_action(ErrorAction::Abort).unwrap();
        assert_eq!(spice.get_error_action().unwrap(), ErrorAction::Abort);
    }
}
