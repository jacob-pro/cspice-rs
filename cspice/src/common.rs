use crate::SpiceString;
use cspice_sys::SpiceChar;
use once_cell::sync::Lazy;

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
    pub fn as_spice_string(&self) -> &'static SpiceString {
        static EQ: Lazy<SpiceString> = Lazy::new(|| SpiceString::from("="));
        static NE: Lazy<SpiceString> = Lazy::new(|| SpiceString::from("<>"));
        static LEQ: Lazy<SpiceString> = Lazy::new(|| SpiceString::from("<="));
        static LT: Lazy<SpiceString> = Lazy::new(|| SpiceString::from("<"));
        static GEQ: Lazy<SpiceString> = Lazy::new(|| SpiceString::from(">="));
        static GT: Lazy<SpiceString> = Lazy::new(|| SpiceString::from(">"));
        match &self {
            ComparisonOperator::EQ => &*EQ,
            ComparisonOperator::NE => &*NE,
            ComparisonOperator::LEQ => &*LEQ,
            ComparisonOperator::LT => &*LT,
            ComparisonOperator::GEQ => &*GEQ,
            ComparisonOperator::GT => &*GT,
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
