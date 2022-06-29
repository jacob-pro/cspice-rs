use crate::string::SpiceString;
use crate::SPICE;
use cspice_sys::{str2et_c, timout_c, SpiceChar, SpiceDouble, SpiceInt};
use once_cell::sync::Lazy;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::DerefMut;

/// Ephemeris Time (time in seconds past the ephemeris epoch J2000) (TDB)
///
/// See https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/FORTRAN/req/time.html#In%20the%20Toolkit%20ET%20Means%20TDB
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ET(pub SpiceDouble);

impl Display for ET {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl ET {
    #[inline]
    pub fn to_jd(&self, spice: SPICE) -> JulianDate {
        static mut PICTUR: Lazy<SpiceString> =
            Lazy::new(|| SpiceString::from("JULIAND.############# ::TDB"));
        let mut buffer = vec![0; 40];
        unsafe {
            timout_c(
                self.0,
                PICTUR.as_mut_ptr(),
                buffer.len() as i32,
                buffer.as_mut_ptr(),
            );
        }
        spice.get_last_error().unwrap();
        JulianDate(SpiceString::from_buffer(buffer).as_str().parse().unwrap())
    }

    #[inline]
    pub fn from_jd(jd: JulianDate, spice: SPICE) -> Self {
        jd.to_et(spice)
    }

    pub fn to_ut<C: Calendar>(&self, spice: SPICE) -> UT<C> {
        let mut buffer = vec![0; 100];
        unsafe {
            timout_c(
                self.0,
                C::timdef_pictur(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        };
        spice.get_last_error().unwrap();
        let output = SpiceString::from_buffer(buffer).to_string();
        println!("{}", output);
        let split: Vec<&str> = output.split(':').collect();
        let year: i32 = if split[0] == "B.C." {
            1 - split[1].trim().parse::<i32>().unwrap()
        } else {
            split[1].trim().parse().unwrap()
        };
        UT {
            year,
            month: split[2].parse().unwrap(),
            day: split[3].parse().unwrap(),
            hour: split[4].parse().unwrap(),
            minute: split[5].parse().unwrap(),
            second: split[6].parse().unwrap(),
            phantom: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct JulianDate(pub SpiceDouble);

impl JulianDate {
    #[inline]
    pub fn to_et(&self, spice: SPICE) -> ET {
        let mut value = SpiceString::from(format!("JDTDB {}", self.0));
        let mut output = 0f64;
        unsafe {
            str2et_c(value.as_mut_ptr(), &mut output);
        }
        spice.get_last_error().unwrap();
        ET(output)
    }

    #[inline]
    pub fn from_et(et: ET, spice: SPICE) -> Self {
        et.to_jd(spice)
    }
}

impl Display for JulianDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UT<T: Calendar> {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: f32,
    phantom: PhantomData<T>,
}

impl<C: Calendar> UT<C> {
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: f32) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            phantom: Default::default(),
        }
    }
}

pub trait Calendar {
    fn timdef_pictur() -> *mut SpiceChar;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mixed;

impl Calendar for Mixed {
    fn timdef_pictur() -> *mut SpiceChar {
        static mut PICTUR: Lazy<SpiceString> =
            Lazy::new(|| SpiceString::from("ERA:YYYY:MM:DD:HR:MN:SC.##### ::TDB ::MCAL"));
        unsafe { PICTUR.deref_mut().as_mut_ptr() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_spice;

    #[test]
    fn test_et_to_jd() {
        let spice = get_test_spice();
        assert_eq!(ET(0f64).to_jd(spice), JulianDate(2451545.0));
    }

    #[test]
    fn test_jd_to_et() {
        let spice = get_test_spice();
        let et = JulianDate(1502273.5).to_et(spice);
        let ut = et.to_ut::<Mixed>(spice);
        assert_eq!(ut, UT::<Mixed>::new(-599, 1, 1, 0, 0, 0.0));
    }
}
