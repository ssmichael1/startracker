use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::wrap_pymodule;

mod pycameraframe;
mod pyhealpix;
mod pyserfile;
mod pystarfinder;

use pycameraframe::*;
use pyserfile::*;
use pystarfinder::*;

#[pymodule]
fn healpix(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pyhealpix::pix2ang_ring, m)?)?;
    m.add_function(wrap_pyfunction!(pyhealpix::ang2pix_ring, m)?)?;
    Ok(())
}

#[pymodule]
fn startracker(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyPixelOrder>()?;
    m.add_class::<PyPixelFormat>()?;
    m.add_class::<PyCameraFrame>()?;
    m.add_class::<PySERFile>()?;
    m.add_class::<PyImageSegment>()?;
    m.add_class::<PyFindStarsOptions>()?;
    m.add_function(wrap_pyfunction!(py_find_stars, m)?)?;
    m.add_wrapped(wrap_pymodule!(healpix))?;
    Ok(())
}
