//! Miscellaneous enums and structures.
use crate::string::{static_spice_str, StaticSpiceStr};
use cspice_sys::SpiceChar;

pub(crate) static SET: StaticSpiceStr = static_spice_str!("SET");
pub(crate) static GET: StaticSpiceStr = static_spice_str!("GET");
pub(crate) static CALENDAR: StaticSpiceStr = static_spice_str!("CALENDAR");

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ComparisonOperator {
    EQ,
    NE,
    LEQ,
    LT,
    GEQ,
    GT,
}

impl ComparisonOperator {
    pub(crate) unsafe fn as_spice_char(&self) -> *mut SpiceChar {
        match &self {
            ComparisonOperator::EQ => static_spice_str!("="),
            ComparisonOperator::NE => static_spice_str!("<>"),
            ComparisonOperator::LEQ => static_spice_str!("<="),
            ComparisonOperator::LT => static_spice_str!("<"),
            ComparisonOperator::GEQ => static_spice_str!(">="),
            ComparisonOperator::GT => static_spice_str!(">"),
        }
        .as_mut_ptr()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub fn as_spice_char(&self) -> SpiceChar {
        (match &self {
            Side::Left => 'L',
            Side::Right => 'R',
        }) as SpiceChar
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum AberrationCorrection {
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

impl AberrationCorrection {
    pub(crate) unsafe fn as_spice_char(&self) -> *mut SpiceChar {
        match &self {
            AberrationCorrection::NONE => static_spice_str!("NONE"),
            AberrationCorrection::LT => static_spice_str!("LT"),
            AberrationCorrection::LT_S => static_spice_str!("LT+S"),
            AberrationCorrection::CN => static_spice_str!("CN"),
            AberrationCorrection::CN_S => static_spice_str!("CN+S"),
            AberrationCorrection::XLT => static_spice_str!("XLT"),
            AberrationCorrection::XLT_S => static_spice_str!("XLT+S"),
            AberrationCorrection::XCN => static_spice_str!("XCN"),
            AberrationCorrection::XCN_S => static_spice_str!("XCN+S"),
        }
        .as_mut_ptr()
    }
}
