//! Functions relating to the Spacecraft and Planet Ephemeris (SPK) subsystem of SPICE.
use crate::common::AberrationCorrection;
use crate::error::get_last_error;
use crate::string::StringParam;
use crate::time::Et;
use crate::vector::Vector3D;
use crate::{spice_unsafe, Error};
use cspice_sys::{spkez_c, spkezp_c, spkezr_c, spkpos_c, SpiceDouble};

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

/// Return the state (position and velocity) of a target body
/// relative to an observing body, optionally corrected for light
/// time (planetary aberration) and stellar aberration.
///
/// See [spkez_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/spkez_c.html).
pub fn easy_reader<'r, R>(
    target: i32,
    et: Et,
    reference_frame: R,
    aberration_correction: AberrationCorrection,
    observing_body: i32,
) -> Result<([SpiceDouble; 6], SpiceDouble), Error>
where
    R: Into<StringParam<'r>>,
{
    let mut pos_vel = [0.0f64; 6];
    let mut light_time = 0.0;
    spice_unsafe!({
        spkez_c(
            target,
            et.0,
            reference_frame.into().as_mut_ptr(),
            aberration_correction.as_spice_char(),
            observing_body,
            pos_vel.as_mut_ptr(),
            &mut light_time,
        )
    });
    get_last_error()?;
    Ok((pos_vel, light_time))
}

/// Return the position of a target body relative to an observing
/// body, optionally corrected for light time (planetary aberration)
/// and stellar aberration.
///
/// See [spkezp_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/spkezp_c.html).
pub fn easy_position<'r, R>(
    target: i32,
    et: Et,
    reference_frame: R,
    aberration_correction: AberrationCorrection,
    observing_body: i32,
) -> Result<(Vector3D, SpiceDouble), Error>
where
    R: Into<StringParam<'r>>,
{
    let mut position = Vector3D::default();
    let mut light_time = 0.0;
    spice_unsafe!({
        spkezp_c(
            target,
            et.0,
            reference_frame.into().as_mut_ptr(),
            aberration_correction.as_spice_char(),
            observing_body,
            position.as_mut_ptr(),
            &mut light_time,
        )
    });
    get_last_error()?;
    Ok((position, light_time))
}

/// Return the state (position and velocity) of a target body
/// relative to an observing body, optionally corrected for light
/// time (planetary aberration) and stellar aberration.
///
/// See [spkezr_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/spkezr_c.html)
pub fn easier_reader<'t, 'r, 'o, T, R, O>(
    target: T,
    et: Et,
    reference_frame: R,
    aberration_correction: AberrationCorrection,
    observing_body: O,
) -> Result<([SpiceDouble; 6], SpiceDouble), Error>
where
    T: Into<StringParam<'t>>,
    R: Into<StringParam<'r>>,
    O: Into<StringParam<'o>>,
{
    let mut pos_vel = [0.0f64; 6];
    let mut light_time = 0.0;
    spice_unsafe!({
        spkezr_c(
            target.into().as_mut_ptr(),
            et.0,
            reference_frame.into().as_mut_ptr(),
            aberration_correction.as_spice_char(),
            observing_body.into().as_mut_ptr(),
            pos_vel.as_mut_ptr(),
            &mut light_time,
        )
    });
    get_last_error()?;
    Ok((pos_vel, light_time))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::load_test_data;
    const EPSILON: f64 = 1e-10;
    const ETS: [Et; 3] = [Et(0.0), Et(3600.0), Et(120000.0)];
    // Test data generated via spiceypy using the above ephemeris times
    const TEST_DATA: [[SpiceDouble; 6]; 3] = [
        [
            -291569.26474221050739f64,
            -266709.18712562322617f64,
            -76099.15410456061363f64,
            0.64353061379157f64,
            -0.66608181544709f64,
            -0.30132283179347f64,
        ],
        [
            -289240.78060919046402f64,
            -269096.44152130186558f64,
            -77180.89871158450842f64,
            0.65006211575479f64,
            -0.66016273764220f64,
            -0.29964267392589f64,
        ],
        [
            -202558.33919326588511f64,
            -333880.37279736995697f64,
            -108450.58380541205406f64,
            0.82840534359059f64,
            -0.44612163419131f64,
            -0.23419745913028f64,
        ],
    ];
    const LTS: [SpiceDouble; 3] = [
        1.3423106094958182f64,
        1.342693954033622f64,
        1.3519329044685606f64,
    ];

    #[test]
    fn moon_earth_spkpos_test() {
        load_test_data();
        for i in 0..3 {
            let (pos, lt) =
                position("moon", ETS[i], "J2000", AberrationCorrection::LT, "earth").unwrap();
            for j in 0..3 {
                assert!((pos[j] - TEST_DATA[i][j]).abs() < EPSILON);
            }
            assert!((lt - LTS[i]).abs() < EPSILON);
        }
    }

    #[test]
    fn moon_earth_spkez_test() {
        load_test_data();
        for i in 0..3 {
            let (pos_vel, lt) =
                easy_reader(301, ETS[i], "J2000", AberrationCorrection::LT, 399).unwrap();
            for j in 0..6 {
                assert!((pos_vel[j] - TEST_DATA[i][j]).abs() < EPSILON);
            }
            assert!((lt - LTS[i]).abs() < EPSILON);
        }
    }

    #[test]
    fn moon_earth_spkezp_test() {
        load_test_data();
        for i in 0..3 {
            let (pos, lt) =
                easy_position(301, ETS[i], "J2000", AberrationCorrection::LT, 399).unwrap();
            for j in 0..3 {
                assert!((pos[j] - TEST_DATA[i][j]).abs() < EPSILON);
            }
            assert!((lt - LTS[i]).abs() < EPSILON);
        }
    }

    #[test]
    fn moon_earth_spkezr_test() {
        load_test_data();
        for i in 0..3 {
            let (pos_vel, lt) =
                easier_reader("moon", ETS[i], "J2000", AberrationCorrection::LT, "earth").unwrap();
            for j in 0..6 {
                assert!((pos_vel[j] - TEST_DATA[i][j]).abs() < EPSILON);
            }
            assert!((lt - LTS[i]).abs() < EPSILON);
        }
    }
}
