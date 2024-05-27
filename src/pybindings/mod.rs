use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

mod pycameraframe;
mod pyserfile;
mod pystarfinder;

use pycameraframe::*;
use pyserfile::*;
use pystarfinder::*;

#[pymodule]
fn startracker(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPixelOrder>()?;
    m.add_class::<PyPixelFormat>()?;
    m.add_class::<PyCameraFrame>()?;
    m.add_class::<PySERFile>()?;
    m.add_class::<PyImageSegment>()?;
    m.add_function(wrap_pyfunction!(py_find_stars, m)?)?;
    Ok(())
}