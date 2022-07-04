//! Miscellaneous enums and structures.
use crate::string::{static_spice_str, StaticSpiceStr};
use cspice_sys::SpiceChar;

pub(crate) static SET: StaticSpiceStr = static_spice_str!("SET");
pub(crate) static GET: StaticSpiceStr = static_spice_str!("GET");
pub(crate) static CALENDAR: StaticSpiceStr = static_spice_str!("CALENDAR");

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ComparisonOperator {
    EQ,
    NE,
    LEQ,
    LT,
    GEQ,
    GT,
}

impl ComparisonOperator {
    pub(crate) fn as_spice_str(&self) -> StaticSpiceStr {
        match &self {
            ComparisonOperator::EQ => static_spice_str!("="),
            ComparisonOperator::NE => static_spice_str!("<>"),
            ComparisonOperator::LEQ => static_spice_str!("<="),
            ComparisonOperator::LT => static_spice_str!("<"),
            ComparisonOperator::GEQ => static_spice_str!(">="),
            ComparisonOperator::GT => static_spice_str!(">"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
