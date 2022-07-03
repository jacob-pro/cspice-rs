/// See [Calendars in timout_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/timout_c.html).
pub trait Calendar {
    fn short_name() -> &'static str;
}

/// Uses the Julian calendar for dates prior to Oct 5, 1582, and the Gregorian calendar for dates
/// after Oct 15, 1582.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mixed;

/// The Gregorian calendar. Dates before the Gregorian calendar's inception in 1582 are defined via
/// extrapolation.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Gregorian;

/// The Julian calendar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Julian;

impl Calendar for Mixed {
    fn short_name() -> &'static str {
        "MCAL"
    }
}

impl Calendar for Gregorian {
    fn short_name() -> &'static str {
        "GCAL"
    }
}

impl Calendar for Julian {
    fn short_name() -> &'static str {
        "JCAL"
    }
}
