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
        static EQ: StaticSpiceStr = static_spice_str!("=");
        static NE: StaticSpiceStr = static_spice_str!("<>");
        static LEQ: StaticSpiceStr = static_spice_str!("<=");
        static LT: StaticSpiceStr = static_spice_str!("<");
        static GEQ: StaticSpiceStr = static_spice_str!(">=");
        static GT: StaticSpiceStr = static_spice_str!(">");
        match &self {
            ComparisonOperator::EQ => EQ,
            ComparisonOperator::NE => NE,
            ComparisonOperator::LEQ => LEQ,
            ComparisonOperator::LT => LT,
            ComparisonOperator::GEQ => GEQ,
            ComparisonOperator::GT => GT,
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
