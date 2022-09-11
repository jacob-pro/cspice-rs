//! The time systems supported by SPICE.
use std::borrow::Cow;

/// See [SPICE Time Subsystem](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html).
pub trait System: Default {
    fn system_name() -> &'static str;
    fn meta_marker(&self) -> Cow<'static, str>;
}

/// Terrestrial Dynamical Time (TDT).
///
/// Note: TDT and TT represent the same time system
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Tdt;

/// Barycentric Dynamical Time (TDB).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Tdb;

/// Coordinated Universal Time (UTC).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Utc {
    pub zone_hours: i8,
    pub zone_minutes: u8,
}

impl System for Tdt {
    fn system_name() -> &'static str {
        "TDT"
    }

    fn meta_marker(&self) -> Cow<'static, str> {
        "TDT".into()
    }
}

impl System for Tdb {
    fn system_name() -> &'static str {
        "TDB"
    }

    fn meta_marker(&self) -> Cow<'static, str> {
        "TDB".into()
    }
}

impl System for Utc {
    fn system_name() -> &'static str {
        "UTC"
    }

    fn meta_marker(&self) -> Cow<'static, str> {
        format!("UTC{:+}:{}", self.zone_hours, self.zone_minutes).into()
    }
}

impl Utc {
    #[inline]
    pub fn new(zone_hours: i8, zone_minutes: u8) -> Self {
        Self {
            zone_hours,
            zone_minutes,
        }
    }

    #[inline]
    pub fn to_zone_seconds(&self) -> i32 {
        let hour_component = self.zone_hours.abs() as i32 * 60 * 60;
        let minute_component = self.zone_minutes as i32 * 60;
        let sum = hour_component + minute_component;
        if self.zone_hours.is_negative() {
            return -sum;
        }
        sum
    }

    /// This will round to the nearest minute.
    #[inline]
    pub fn from_zone_seconds(seconds: i32) -> Self {
        let abs = seconds.abs();
        let hours = abs / 3600;
        let minutes = ((abs % 3600) as f32 / 60.0).round();
        let hours = if seconds.is_negative() { -hours } else { hours };
        Self {
            zone_hours: hours as i8,
            zone_minutes: minutes as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::time::system::Utc;

    #[test]
    fn test_utc_from_seconds() {
        let utc = Utc::from_zone_seconds(9000);
        assert_eq!(utc, Utc::new(2, 30));

        let utc = Utc::from_zone_seconds(-9000);
        assert_eq!(utc, Utc::new(-2, 30));

        let utc = Utc::from_zone_seconds(-9001);
        assert_eq!(utc, Utc::new(-2, 30));

        let utc = Utc::from_zone_seconds(-9050);
        assert_eq!(utc, Utc::new(-2, 31));
    }

    #[test]
    fn test_utc_to_seconds() {
        let utc = Utc::new(2, 30);
        assert_eq!(utc.to_zone_seconds(), 9000);

        let utc = Utc::new(-2, 30);
        assert_eq!(utc.to_zone_seconds(), -9000);
    }
}
