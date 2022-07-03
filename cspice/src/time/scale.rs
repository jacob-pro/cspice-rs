/// See [SPICE Time Subsystem](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html).
pub trait Scale {
    fn name() -> &'static str;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tdt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tdb;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Utc;

impl Scale for Tdt {
    fn name() -> &'static str {
        "TDT"
    }
}

impl Scale for Tdb {
    fn name() -> &'static str {
        "TDB"
    }
}

impl Scale for Utc {
    fn name() -> &'static str {
        "UTC"
    }
}
