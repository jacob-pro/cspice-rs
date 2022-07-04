use crate::common::{CALENDAR, GET, SET};
use crate::error::get_last_error;
use crate::string::SpiceStr;
use crate::time::calendar::Calendar;
use crate::time::julian_date::JulianDate;
use crate::time::system::System;
use crate::time::{set_default_calendar, Et};
use crate::{spice_unsafe, SpiceString};
use cspice_sys::{timdef_c, timout_c, SpiceInt};
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DateTime<T: Calendar, S: System> {
    pub year: i16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: f32,
    pub system: S,
    calendar: PhantomData<T>,
}

impl<C: Calendar, S: System> DateTime<C, S> {
    #[inline]
    pub fn new(
        year: i16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: f32,
        system: S,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            system,
            calendar: Default::default(),
        }
    }

    /// Convert an Ephemeris Time (TDB) to a DateTime.
    #[inline]
    pub fn from_et(et: Et, system: S) -> Self {
        let pictur = SpiceString::from(format!(
            "ERA:YYYY:MM:DD:HR:MN:SC.##### ::{} ::{}",
            system.meta_marker(),
            C::short_name()
        ));
        let mut buffer = [0; 100];
        spice_unsafe!({
            timout_c(
                et.0,
                pictur.as_mut_ptr(),
                buffer.len() as SpiceInt,
                buffer.as_mut_ptr(),
            );
        });
        get_last_error().unwrap();
        let output = SpiceStr::from_buffer(&buffer);
        let cow = output.as_str();
        let split: Vec<&str> = cow.split(':').collect();
        let year: i16 = if split[0] == "B.C." {
            1 - split[1].trim().parse::<i16>().unwrap()
        } else {
            split[1].trim().parse().unwrap()
        };
        DateTime::new(
            year,
            split[2].parse().unwrap(),
            split[3].parse().unwrap(),
            split[4].parse().unwrap(),
            split[5].parse().unwrap(),
            split[6].parse().unwrap(),
            system,
        )
    }
}

impl<C: Calendar, S: System> From<Et> for DateTime<C, S> {
    #[inline]
    fn from(et: Et) -> Self {
        DateTime::from_et(et, S::default())
    }
}

impl<C: Calendar, S: System> From<DateTime<C, S>> for Et {
    /// Convert a DateTime to Ephemeris Time (TDB)
    #[inline]
    fn from(dt: DateTime<C, S>) -> Self {
        // Get default calendar setting
        let mut original_cal = [0; 12];
        spice_unsafe!({
            timdef_c(
                GET.as_mut_ptr(),
                CALENDAR.as_mut_ptr(),
                original_cal.len() as SpiceInt,
                original_cal.as_mut_ptr(),
            );
        });
        get_last_error().unwrap();
        let year = if dt.year > 0 {
            dt.year.to_string()
        } else {
            format!("{} BC", dt.year.abs() + 1)
        };
        let date = format!(
            "{year}-{}-{} {}:{}:{} {}",
            dt.month,
            dt.day,
            dt.hour,
            dt.minute,
            dt.second,
            dt.system.meta_marker(),
        );
        set_default_calendar::<C>();
        let et = Et::from_string(date).unwrap();
        // Restore default calendar
        spice_unsafe!({
            timdef_c(
                SET.as_mut_ptr(),
                CALENDAR.as_mut_ptr(),
                0,
                original_cal.as_mut_ptr(),
            );
        });
        get_last_error().unwrap();
        et
    }
}

impl<C: Calendar, S: System> From<JulianDate<S>> for DateTime<C, S> {
    #[inline]
    fn from(jd: JulianDate<S>) -> Self {
        DateTime::from(Et::from(jd))
    }
}

impl<C: Calendar, S: System> Display for DateTime<C, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{} {}:{}:{} {} {}",
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.system.meta_marker(),
            C::short_name()
        )
    }
}

#[cfg(feature = "chrono")]
impl From<chrono::DateTime<chrono::FixedOffset>>
    for DateTime<super::calendar::Gregorian, super::system::Utc>
{
    fn from(c: chrono::DateTime<chrono::FixedOffset>) -> Self {
        use chrono::{Datelike, Timelike};
        let seconds = c.second() as f32 + c.nanosecond() as f32 / 1_000_000.0;
        DateTime::new(
            c.year() as i16,
            c.month() as u8,
            c.day() as u8,
            c.hour() as u8,
            c.minute() as u8,
            seconds,
            super::system::Utc::from_zone_seconds(c.timezone().local_minus_utc()),
        )
    }
}

#[cfg(feature = "chrono")]
impl From<DateTime<super::calendar::Gregorian, super::system::Utc>>
    for chrono::DateTime<chrono::FixedOffset>
{
    fn from(t: DateTime<super::calendar::Gregorian, super::system::Utc>) -> Self {
        use chrono::TimeZone;
        let ns = t.second.fract() * 1_000_000_f32;
        chrono::FixedOffset::east(t.system.to_zone_seconds())
            .ymd(t.year as i32, t.month as u32, t.day as u32)
            .and_hms_nano(
                t.hour as u32,
                t.minute as u32,
                t.second.floor() as u32,
                ns as u32,
            )
    }
}
