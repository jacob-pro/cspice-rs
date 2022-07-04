//! Functions relating to the Spacecraft and Planet Ephemeris (SPK) subsystem of SPICE.
use crate::common::AberrationCorrection;
use crate::error::get_last_error;
use crate::string::StringParam;
use crate::time::Et;
use crate::vector::Vector3D;
use crate::{spice_unsafe, Error};
use cspice_sys::{spkpos_c, SpiceDouble};

/// Return the position of a target body relative to an observing body, optionally corrected for
/// light time (planetary aberration) and stellar aberration.
///
/// See [spkpos_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/spkpos_c.html).
pub fn position<'t, 'r, 'o, T, R, O>(
    target: T,
    et: Et,
    reference_frame: R,
    aberration_correction: AberrationCorrection,
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
            aberration_correction.as_spice_char(),
            observing_body.into().as_mut_ptr(),
            position.as_mut_ptr(),
            &mut light_time,
        )
    });
    get_last_error()?;
    Ok((position, light_time))
}
