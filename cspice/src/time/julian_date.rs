use crate::time::date_time::DateTime;
use crate::time::{Calendar, Scale, ET};
use crate::SPICE;
use cspice_sys::SpiceDouble;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JulianDate<S: Scale> {
    pub value: SpiceDouble,
    scale: PhantomData<S>,
}

impl<S: Scale> JulianDate<S> {
    #[inline]
    pub fn new(jd: SpiceDouble) -> Self {
        Self {
            value: jd,
            scale: Default::default(),
        }
    }

    /// Convert the Julian Date to Ephemeris Time (TDB)
    #[inline]
    pub fn to_et(&self, spice: SPICE) -> ET {
        spice
            .string_to_et(format!("JD {} {}", S::name(), self.value))
            .unwrap()
    }

    /// Equivalent to [ET::to_julian_date()]
    #[inline]
    pub fn from_et(et: ET, spice: SPICE) -> Self {
        et.to_julian_date(spice)
    }

    #[inline]
    pub fn to_date_time<C: Calendar>(&self, spice: SPICE) -> DateTime<C, S> {
        self.to_et(spice).to_date_time(spice)
    }

    #[inline]
    pub fn from_date_time<C: Calendar>(date_time: DateTime<C, S>, spice: SPICE) -> Self {
        date_time.to_et(spice).to_julian_date(spice)
    }
}

impl<S: Scale> Display for JulianDate<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "JD {} {}", S::name(), self.value)
    }
}
