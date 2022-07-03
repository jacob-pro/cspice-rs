use crate::SpiceString;
use once_cell::sync::Lazy;

pub static SET: Lazy<SpiceString> = Lazy::new(|| SpiceString::from("SET"));
pub static GET: Lazy<SpiceString> = Lazy::new(|| SpiceString::from("GET"));
pub static CALENDAR: Lazy<SpiceString> = Lazy::new(|| SpiceString::from("CALENDAR"));
