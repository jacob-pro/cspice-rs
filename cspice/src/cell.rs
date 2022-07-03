use crate::string::StringParam;
use crate::{Error, Spice};
use cspice_sys::{
    SpiceBoolean, SpiceChar, SpiceDouble, SpiceInt, _SpiceDataType_SPICE_CHR,
    _SpiceDataType_SPICE_DP, _SpiceDataType_SPICE_INT, appndc_c, appndd_c, appndi_c, card_c,
    copy_c, scard_c, SPICEFALSE, SPICETRUE, SPICE_CELL_CTRLSZ,
};
use std::ffi::c_void;

pub trait CellType {}

impl CellType for SpiceDouble {}
impl CellType for SpiceInt {}
impl CellType for SpiceChar {}

pub struct Cell<T: CellType> {
    cell: cspice_sys::SpiceCell,
    #[allow(dead_code)]
    data: Vec<T>,
}

impl<T: CellType> Cell<T> {
    pub fn get_cell(&mut self) -> *mut cspice_sys::SpiceCell {
        &mut self.cell
    }

    /// Set the cardinality of a cell.
    ///
    /// See [scard_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/scard_c.html).
    pub fn set_cardinality(&mut self, cardinality: SpiceInt, spice: Spice) -> Result<(), Error> {
        unsafe {
            scard_c(cardinality, self.get_cell());
        }
        spice.get_last_error()
    }

    /// Return the size (maximum cardinality) of a SPICE cell.
    ///
    /// See [size_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/size_c.html)
    pub fn get_size(&mut self, spice: Spice) -> Result<(), Error> {
        unsafe {
            card_c(self.get_cell());
        }
        spice.get_last_error()
    }

    /// Return the cardinality (current number of elements) in a cell.
    ///
    /// See [card_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/card_c.html).
    pub fn get_cardinality(&mut self, spice: Spice) -> Result<(), Error> {
        unsafe {
            card_c(self.get_cell());
        }
        spice.get_last_error()
    }

    /// Copy the contents of a SpiceCell of any data type to another cell of the same type.
    ///
    /// See [copy_c](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/copy_c.html).
    pub fn copy(&mut self, dest: &mut Cell<T>, spice: Spice) -> Result<(), Error> {
        unsafe {
            copy_c(self.get_cell(), dest.get_cell());
        }
        spice.get_last_error()
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
    pub fn append(&mut self, item: SpiceDouble, spice: Spice) -> Result<(), Error> {
        unsafe {
            appndd_c(item, self.get_cell());
        }
        spice.get_last_error()
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
    pub fn append(&mut self, item: SpiceInt, spice: Spice) -> Result<(), Error> {
        unsafe {
            appndi_c(item, self.get_cell());
        }
        spice.get_last_error()
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
    pub fn append<'s, S: Into<StringParam<'s>>>(
        &mut self,
        item: S,
        spice: Spice,
    ) -> Result<(), Error> {
        unsafe {
            appndc_c(item.into().as_mut_ptr(), self.get_cell());
        }
        spice.get_last_error()
    }
}
