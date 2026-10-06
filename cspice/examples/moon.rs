use cspice::common::AberrationCorrection;
use cspice::data::furnish;
use cspice::spk::easier_reader;
use cspice::time::Et;

fn main() -> Result<(), cspice::Error> {
    // Load kernels, e.g. from https://naif.jpl.nasa.gov/pub/naif/generic_kernels/
    furnish("naif0012.tls")?; // leap seconds kernel, required for time conversion
    furnish("de432s.bsp")?; // planetary ephemeris

    // Parse a UTC string into ephemeris time, seconds past J2000 TDB
    let et = Et::from_string("2026-10-05T12:00:00")?;

    // State of the Moon relative to Earth in the J2000 frame, corrected for light time
    let (state, light_time) =
        easier_reader("MOON", et, "J2000", AberrationCorrection::LT, "EARTH")?;

    println!(
        "Moon wrt Earth: x = {} km, y = {} km, z = {} km, one-way light time = {} s",
        state.position.x, state.position.y, state.position.z, light_time
    );
    Ok(())
}
