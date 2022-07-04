use crate::string::{static_spice_str, StaticSpiceStr, StringParam};
use crate::time::Et;
use crate::vector::Vector3D;
use crate::{spice_unsafe, Error, Spice};
use cspice_sys::{spkpos_c, SpiceDouble};

#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AberrationCorrectionFlag {
    NONE,
    LT,
    LT_S,
    CN,
    CN_S,
    XLT,
    XLT_S,
    XCN,
    XCN_S,
}

impl AberrationCorrectionFlag {
    pub(crate) fn as_spice_str(&self) -> StaticSpiceStr {
        match &self {
            AberrationCorrectionFlag::NONE => static_spice_str!("NONE"),
            AberrationCorrectionFlag::LT => static_spice_str!("LT"),
            AberrationCorrectionFlag::LT_S => static_spice_str!("LT+S"),
            AberrationCorrectionFlag::CN => static_spice_str!("CN"),
            AberrationCorrectionFlag::CN_S => static_spice_str!("CN+S"),
            AberrationCorrectionFlag::XLT => static_spice_str!("XLT"),
            AberrationCorrectionFlag::XLT_S => static_spice_str!("XLT+S"),
            AberrationCorrectionFlag::XCN => static_spice_str!("XCN"),
            AberrationCorrectionFlag::XCN_S => static_spice_str!("XCN+S"),
        }
    }
}

/// Functions relating to the Spacecraft and Planet Ephemeris (SPK) subsystem of SPICE.
impl Spice {
    /// Return the position of a target body relative to an observing body, optionally corrected for
    /// light time (planetary aberration) and stellar aberration.
    ///
    /// See [spkpos_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/spkpos_c.html).
    pub fn spk_position<'t, 'r, 'o, T, R, O>(
        &self,
        target: T,
        et: Et,
        reference_frame: R,
        aberration_correction: &AberrationCorrectionFlag,
        observing_body: O,
    ) -> Result<(Vector3D, SpiceDouble), Error>
    where
        T: Into<StringParam<'t>>,
        R: Into<StringParam<'r>>,
        O: Into<StringParam<'o>>,
    {
        let mut position = Vector3D::default();
        let mut light_time = 0.0;
        spice_unsafe!({
            spkpos_c(
                target.into().as_mut_ptr(),
                et.0,
                reference_frame.into().as_mut_ptr(),
                aberration_correction.as_spice_str().as_mut_ptr(),
                observing_body.into().as_mut_ptr(),
                position.as_mut_ptr(),
                &mut light_time,
            )
        });
        Spice::get_last_error()?;
        Ok((position, light_time))
    }
}
