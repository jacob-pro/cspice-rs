//! Functions for working with SPICE Cells.
use crate::common::{ComparisonOperator, Side};
use crate::error::get_last_error;
use crate::string::StringParam;
use crate::{spice_unsafe, Error};
use cspice_sys::{
    _SpiceDataType_SPICE_CHR, _SpiceDataType_SPICE_DP, _SpiceDataType_SPICE_INT, appndc_c,
    appndd_c, appndi_c, card_c, copy_c, scard_c, wncard_c, wncomd_c, wncond_c, wndifd_c, wnelmd_c,
    wnexpd_c, wnextd_c, wnfetd_c, wnfild_c, wnfltd_c, wnincd_c, wninsd_c, wnintd_c, wnreld_c,
    wnsumd_c, wnunid_c, wnvald_c, SpiceBoolean, SpiceChar, SpiceDouble, SpiceInt, SPICEFALSE,
    SPICETRUE, SPICE_CELL_CTRLSZ,
};
use std::ffi::c_void;

/// A type that can be used in a SPICE Cell.
pub trait CellType {}

impl CellType for SpiceDouble {}
impl CellType for SpiceInt {}
impl CellType for SpiceChar {}

/// A Rust wrapper around a SpiceCell and its data.
pub struct Cell<T: CellType> {
    cell: cspice_sys::SpiceCell,
    #[allow(dead_code)]
    data: Vec<T>,
}

impl<T: CellType> Cell<T> {
    /// Access the internal CSPICE Cell structure.
    pub fn as_mut_cell(&mut self) -> *mut cspice_sys::SpiceCell {
        &mut self.cell
    }

    /// Set the cardinality of a cell.
    ///
    /// See [scard_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/scard_c.html).
    pub fn set_cardinality(&mut self, cardinality: usize) -> Result<(), Error> {
        spice_unsafe!({
            scard_c(cardinality as SpiceInt, self.as_mut_cell());
        });
        get_last_error()
    }

    /// Return the size (maximum cardinality) of a SPICE cell.
    ///
    /// See [size_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/size_c.html)
    pub fn get_size(&mut self) -> Result<usize, Error> {
        let out = spice_unsafe!({ card_c(self.as_mut_cell()) });
        get_last_error()?;
        Ok(out as usize)
    }

    /// Return the cardinality (current number of elements) in a cell.
    ///
    /// See [card_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/card_c.html).
    pub fn get_cardinality(&mut self) -> Result<usize, Error> {
        let out = spice_unsafe!({ card_c(self.as_mut_cell()) });
        get_last_error()?;
        Ok(out as usize)
    }

    /// Copy the contents of a SpiceCell of any data type to another cell of the same type.
    ///
    /// See [copy_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/copy_c.html).
    pub fn copy(&mut self, dest: &mut Cell<T>) -> Result<(), Error> {
        spice_unsafe!({
            copy_c(self.as_mut_cell(), dest.as_mut_cell());
        });
        get_last_error()
    }
}

impl Cell<SpiceDouble> {
    /// Creates a SPICEDOUBLE_CELL
    ///
    /// See [Declaring and Initializing Cells](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/cells.html#Declaring%20and%20Initializing%20Cells)
    pub fn new_double(size: usize) -> Self {
        let mut data = vec![0.0; SPICE_CELL_CTRLSZ as usize + size];
        let cell = cspice_sys::SpiceCell {
            dtype: _SpiceDataType_SPICE_DP,
            length: 0,
            size: size as SpiceInt,
            card: 0,
            isSet: SPICETRUE as SpiceBoolean,
            adjust: SPICEFALSE as SpiceBoolean,
            init: SPICEFALSE as SpiceBoolean,
            base: data.as_mut_ptr() as *mut c_void,
            data: data[SPICE_CELL_CTRLSZ as usize..].as_mut_ptr() as *mut c_void,
        };
        Self { cell, data }
    }

    /// Append an item to a double precision cell
    ///
    /// See [appndd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/appndd_c.html)
    pub fn append(&mut self, item: SpiceDouble) -> Result<(), Error> {
        spice_unsafe!({
            appndd_c(item, self.as_mut_cell());
        });
        get_last_error()
    }
}

impl Cell<SpiceInt> {
    /// Creates a SPICEINT_CELL
    ///
    /// See [Declaring and Initializing Cells](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/cells.html#Declaring%20and%20Initializing%20Cells)
    pub fn new_int(size: usize) -> Self {
        let mut data = vec![0; SPICE_CELL_CTRLSZ as usize + size];
        let cell = cspice_sys::SpiceCell {
            dtype: _SpiceDataType_SPICE_INT,
            length: 0,
            size: size as SpiceInt,
            card: 0,
            isSet: SPICETRUE as SpiceBoolean,
            adjust: SPICEFALSE as SpiceBoolean,
            init: SPICEFALSE as SpiceBoolean,
            base: data.as_mut_ptr() as *mut c_void,
            data: data[SPICE_CELL_CTRLSZ as usize..].as_mut_ptr() as *mut c_void,
        };
        Self { cell, data }
    }

    /// Append an item to an integer cell
    ///
    /// See [appndi_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/appndi_c.html)
    pub fn append(&mut self, item: SpiceInt) -> Result<(), Error> {
        spice_unsafe!({
            appndi_c(item, self.as_mut_cell());
        });
        get_last_error()
    }
}

impl Cell<SpiceChar> {
    /// Creates a SPICECHAR_CELL
    ///
    /// See [Character Cells](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/cells.html#Character%20Cells)
    pub fn new_char(size: usize, length: usize) -> Self {
        let data_len = (SPICE_CELL_CTRLSZ as usize + size) as usize * length;
        let start_index = SPICE_CELL_CTRLSZ as usize * length;
        let mut data = vec![0; data_len];
        let cell = cspice_sys::SpiceCell {
            dtype: _SpiceDataType_SPICE_CHR,
            length: length as SpiceInt,
            size: size as SpiceInt,
            card: 0,
            isSet: SPICETRUE as SpiceBoolean,
            adjust: SPICEFALSE as SpiceBoolean,
            init: SPICEFALSE as SpiceBoolean,
            base: data.as_mut_ptr() as *mut c_void,
            data: data[start_index..].as_mut_ptr() as *mut c_void,
        };
        Self { cell, data }
    }

    /// Append an item to a character cell
    ///
    /// See [appndc_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/appndc_c.html)
    pub fn append<'s, S: Into<StringParam<'s>>>(&mut self, item: S) -> Result<(), Error> {
        spice_unsafe!({
            appndc_c(item.into().as_mut_ptr(), self.as_mut_cell());
        });
        get_last_error()
    }
}

/// Summary of a double precision window.
///
/// Returned from [Cell::window_summarize()]
#[derive(Debug, Clone, PartialEq)]
pub struct WindowSummary {
    pub total_measure_of_intervals: SpiceDouble,
    pub average_measure: SpiceDouble,
    pub standard_deviation: SpiceDouble,
    pub shortest_interval_index: usize,
    pub longest_interval_index: usize,
}

/// Window specific functions
impl Cell<SpiceDouble> {
    /// Return the cardinality (number of intervals) of a double precision window.
    ///
    /// See [wncard_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wncard_c.html).
    pub fn window_cardinality(&mut self) -> Result<SpiceInt, Error> {
        let out = spice_unsafe!({ wncard_c(self.as_mut_cell()) });
        get_last_error()?;
        Ok(out)
    }

    /// Determine the complement of a double precision window with respect to a specified interval.
    ///
    /// See [wncomd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wncomd_c.html).
    pub fn window_compliment(
        &mut self,
        left: SpiceDouble,
        right: SpiceDouble,
        output: &mut Cell<SpiceDouble>,
    ) -> Result<(), Error> {
        spice_unsafe!({
            wncomd_c(left, right, self.as_mut_cell(), output.as_mut_cell());
        });
        get_last_error()
    }

    /// Contract each of the intervals of a double precision window.
    ///
    /// See [wncond_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wncond_c.html).
    pub fn window_contract(&mut self, left: SpiceDouble, right: SpiceDouble) -> Result<(), Error> {
        spice_unsafe!({
            wncond_c(left, right, self.as_mut_cell());
        });
        get_last_error()
    }

    /// Place the difference of two double precision windows into a third window.
    ///
    /// See [wndifd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wndifd_c.html).
    pub fn window_difference(
        &mut self,
        other: &mut Cell<SpiceDouble>,
        output: &mut Cell<SpiceDouble>,
    ) -> Result<(), Error> {
        spice_unsafe!({
            wndifd_c(
                self.as_mut_cell(),
                other.as_mut_cell(),
                output.as_mut_cell(),
            );
        });
        get_last_error()
    }

    /// Determine whether a point is an element of a double precision window
    ///
    /// See [wnelmd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnelmd_c.html).
    pub fn window_contains_element(&mut self, point: SpiceDouble) -> Result<bool, Error> {
        let out = spice_unsafe!({ wnelmd_c(point, self.as_mut_cell()) });
        get_last_error()?;
        Ok(out == SPICETRUE as SpiceBoolean)
    }

    /// Expand each of the intervals of a double precision window
    ///
    /// See [wnexpd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnexpd_c.html).
    pub fn window_expand(&mut self, left: SpiceDouble, right: SpiceDouble) -> Result<(), Error> {
        spice_unsafe!({
            wnexpd_c(left, right, self.as_mut_cell());
        });
        get_last_error()
    }

    /// Extract the left or right endpoints from a double precision window.
    ///
    /// See [wnextd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnextd_c.html).
    pub fn window_extract(&mut self, side: Side) -> Result<(), Error> {
        spice_unsafe!({
            wnextd_c(side.as_spice_char(), self.as_mut_cell());
        });
        get_last_error()
    }

    /// Fetch a particular interval from a double precision window.
    ///
    /// See [wnfetd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnfetd_c.html).
    pub fn window_interval(&mut self, n: usize) -> Result<(SpiceDouble, SpiceDouble), Error> {
        let (mut left, mut right) = (0.0, 0.0);
        spice_unsafe!({
            wnfetd_c(self.as_mut_cell(), n as SpiceInt, &mut left, &mut right);
        });
        get_last_error()?;
        Ok((left, right))
    }

    /// Fill small gaps between adjacent intervals of a double precision window.
    ///
    /// See [wnfild_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnfild_c.html).
    pub fn window_fill(&mut self, small_gap: SpiceDouble) -> Result<(), Error> {
        spice_unsafe!({
            wnfild_c(small_gap, self.as_mut_cell());
        });
        get_last_error()
    }

    /// Filter (remove) small intervals from a double precision window.
    ///
    /// See [wnfltd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnfltd_c.html).
    pub fn window_filter(&mut self, small_interval: SpiceDouble) -> Result<(), Error> {
        spice_unsafe!({
            wnfltd_c(small_interval, self.as_mut_cell());
        });
        get_last_error()
    }

    /// Determine whether an interval is included in a double precision window.
    ///
    /// See [wnincd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnincd_c.html).
    pub fn window_contains_interval(
        &mut self,
        left: SpiceDouble,
        right: SpiceDouble,
    ) -> Result<bool, Error> {
        let out = spice_unsafe!({ wnincd_c(left, right, self.as_mut_cell()) });
        get_last_error()?;
        Ok(out == SPICETRUE as SpiceBoolean)
    }

    /// Insert an interval into a double precision window.
    ///
    /// See [wninsd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wninsd_c.html).
    pub fn window_insert_interval(
        &mut self,
        left: SpiceDouble,
        right: SpiceDouble,
    ) -> Result<(), Error> {
        spice_unsafe!({ wninsd_c(left, right, self.as_mut_cell()) });
        get_last_error()
    }

    /// Place the intersection of two double precision windows into a third window.
    ///
    /// See [wnintd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnintd_c.html).
    pub fn window_intersect(
        &mut self,
        other: &mut Cell<SpiceDouble>,
        output: &mut Cell<SpiceDouble>,
    ) -> Result<(), Error> {
        spice_unsafe!({
            wnintd_c(
                self.as_mut_cell(),
                other.as_mut_cell(),
                output.as_mut_cell(),
            )
        });
        get_last_error()
    }

    /// Compare two double precision windows.
    ///
    /// See [wnreld_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnreld_c.html).
    pub fn window_compare(
        &mut self,
        comparison_op: ComparisonOperator,
        other: &mut Cell<SpiceDouble>,
    ) -> Result<bool, Error> {
        let out = spice_unsafe!({
            wnreld_c(
                self.as_mut_cell(),
                comparison_op.as_spice_str().as_mut_ptr(),
                other.as_mut_cell(),
            )
        });
        get_last_error()?;
        Ok(out == SPICETRUE as SpiceBoolean)
    }

    /// Summarize the contents of a double precision window.
    ///
    /// See [wnsumd_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnsumd_c.html).
    pub fn window_summarize(&mut self) -> Result<WindowSummary, Error> {
        let (mut meas, mut avg, mut stddev) = (0.0, 0.0, 0.0);
        let (mut idxsml, mut idxlon) = (0, 0);
        spice_unsafe!({
            wnsumd_c(
                self.as_mut_cell(),
                &mut meas,
                &mut avg,
                &mut stddev,
                &mut idxsml,
                &mut idxlon,
            )
        });
        get_last_error()?;
        Ok(WindowSummary {
            total_measure_of_intervals: meas,
            average_measure: avg,
            standard_deviation: stddev,
            shortest_interval_index: idxsml as usize,
            longest_interval_index: idxlon as usize,
        })
    }

    /// Place the union of two double precision windows into a third window.
    ///
    /// See [wnunid_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnunid_c.html).
    pub fn window_union(
        &mut self,
        other: &mut Cell<SpiceDouble>,
        output: &mut Cell<SpiceDouble>,
    ) -> Result<(), Error> {
        spice_unsafe!({
            wnunid_c(
                self.as_mut_cell(),
                other.as_mut_cell(),
                output.as_mut_cell(),
            );
        });
        get_last_error()
    }

    /// Form a valid double precision window from the contents of a window array.
    ///
    /// See [wnvald_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/wnvald_c.html).
    pub fn window_validate(&mut self, size: usize, n: usize) -> Result<(), Error> {
        spice_unsafe!({ wnvald_c(size as SpiceInt, n as SpiceInt, self.as_mut_cell()) });
        get_last_error()
    }
}
