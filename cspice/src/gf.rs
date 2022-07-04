//! Geometry Finder functions.

use crate::cell::Window;
use crate::common::AberrationCorrection;
use crate::error::get_last_error;
use crate::string::StaticSpiceStr;
use crate::string::{static_spice_str, StringParam};
use crate::{spice_unsafe, Error};
use cspice_sys::{gfsep_c, SpiceChar, SpiceDouble, SpiceInt};

#[derive(Copy, Clone, Debug)]
pub enum Shape {
    Sphere,
    Point,
}

impl Shape {
    pub(crate) unsafe fn as_spice_char(&self) -> *mut SpiceChar {
        match &self {
            Shape::Sphere => static_spice_str!("SPHERE"),
            Shape::Point => static_spice_str!("POINT"),
        }
        .as_mut_ptr()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RelationalOperator {
    GT,
    EQ,
    LT,
    AbsMax,
    AbsMin,
    LocalMax,
    LocalMin,
}

impl RelationalOperator {
    pub(crate) unsafe fn as_spice_char(&self) -> *mut SpiceChar {
        match &self {
            RelationalOperator::GT => static_spice_str!(">"),
            RelationalOperator::EQ => static_spice_str!("="),
            RelationalOperator::LT => static_spice_str!("<"),
            RelationalOperator::AbsMax => static_spice_str!("ABSMAX"),
            RelationalOperator::AbsMin => static_spice_str!("ABSMIN"),
            RelationalOperator::LocalMax => static_spice_str!("LOCMAX"),
            RelationalOperator::LocalMin => static_spice_str!("LOCMIN"),
        }
        .as_mut_ptr()
    }
}

/// Determine time intervals when the angular separation between the position vectors of two target
/// bodies relative to an observer satisfies a numerical relationship.
///
/// See [gfsep_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/gfsep_c.html)
#[allow(clippy::too_many_arguments)]
pub fn separation_search<'b1, 'f1, 'b2, 'f2, 'o, B1, F1, B2, F2, O>(
    body1: B1,
    shape1: Shape,
    frame1: F1,
    body2: B2,
    shape2: Shape,
    frame2: F2,
    aberration_correction: AberrationCorrection,
    observing_body: O,
    relational_operator: RelationalOperator,
    refval: SpiceDouble,
    adjust: SpiceDouble,
    step_size: SpiceDouble,
    intervals: usize,
    confine: &mut Window,
    output: &mut Window,
) -> Result<(), Error>
where
    B1: Into<StringParam<'b1>>,
    F1: Into<StringParam<'f1>>,
    B2: Into<StringParam<'b2>>,
    F2: Into<StringParam<'f2>>,
    O: Into<StringParam<'o>>,
{
    spice_unsafe!({
        gfsep_c(
            body1.into().as_mut_ptr(),
            shape1.as_spice_char(),
            frame1.into().as_mut_ptr(),
            body2.into().as_mut_ptr(),
            shape2.as_spice_char(),
            frame2.into().as_mut_ptr(),
            aberration_correction.as_spice_char(),
            observing_body.into().as_mut_ptr(),
            relational_operator.as_spice_char(),
            refval,
            adjust,
            step_size,
            intervals as SpiceInt,
            confine.as_mut_cell(),
            output.as_mut_cell(),
        );
    });
    get_last_error()?;
    Ok(())
}
