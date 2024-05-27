use pyo3::prelude::*;

use crate::serfile::*;
use crate::pybindings::PyCameraFrame;

#[pyclass(name="SERFile")]
#[derive(Clone, Debug)]
pub struct PySERFile {
    inner: SERFile,
}

#[pymethods]
impl PySERFile {
    #[new]
    fn new(filename: String) -> PyResult<Self> {
        Ok(
        PySERFile {
            inner: {
                match SERFile::new(&filename) {
                    Ok(f) => f,
                    Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())),
                }
            }
        })
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    #[getter]
    fn get_filename(&self) -> String {
        self.inner.fname.clone()
    }

    #[getter]
    fn get_frame_count(&self) -> usize {
        self.inner.frames.len()
    }

    fn get_frame(&self, index: usize) -> PyResult<PyCameraFrame> {
        match self.inner.frames.get(index) {
            Some(frame) => Ok(PyCameraFrame { inner: frame.clone() }),
            None => Err(PyErr::new::<pyo3::exceptions::PyIndexError, _>("Index out of bounds")),
        }
    }

    fn __getitem__(&self, index: usize) -> PyResult<PyCameraFrame> {
        self.get_frame(index)
    }

    fn __len__(&self) -> usize {
        self.inner.frames.len()
    }

    #[getter]
    fn instrument(&self) -> String {
        self.inner.instrument.clone()
    }

    #[getter]
    fn observer(&self) -> String {
        self.inner.observer.clone()
    }

    #[getter]
    fn telescope(&self) -> String {
        self.inner.telescope.clone()
    }

    #[getter]
    fn bit_depth(&self) -> usize {
        self.inner.bit_depth
    }

    #[getter]
    fn width(&self) -> usize {
        self.inner.width
    }   

    #[getter]
    fn height(&self) -> usize {
        self.inner.height
    }

}