use pyo3::prelude::*;
use numpy as np;

use crate::starfinder::{Segment, find_stars};

#[pyclass(name="ImageSegment")]
#[derive(Clone, Debug)]
pub struct PyImageSegment {
    inner: Segment,
}

#[pymethods]
impl PyImageSegment {
    #[new]
    fn new() -> Self {
        PyImageSegment {
            inner: Segment::new(),
        }
    }

    #[getter]
    fn get_indices(&self) -> Vec<usize> {
        self.inner.indices.clone()
    }

    #[getter]
    fn get_rowcol(&self) -> Vec<(usize, usize)> {
        self.inner.rowcol.clone()
    }

    #[getter]
    fn get_centroid(&self) -> (f64, f64) {
        self.inner.centroid
    }   

    #[getter]
    fn get_mass(&self) -> f64 {
        self.inner.mass
    }

    #[getter]
    fn get_count(&self) -> usize {
        self.inner.indices.len()
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }   
}

fn find_stars_py<T> (pix: &[T], rows: usize, cols: usize) -> Vec<PyImageSegment>
    where T: Sized + Clone + Copy + Into<i64>
            + Into<f64> + Into<i32>
            + std::cmp::Ord
            + std::ops::Add<Output = T>
{
    let segments = find_stars(&pix, rows, cols);
    let mut py_segments = Vec::<PyImageSegment>::new();
    segments.iter().for_each(|segment| {
        let py_segment = PyImageSegment {
            inner: segment.clone(),
        };
        py_segments.push(py_segment);
    });
    py_segments
}


#[pyfunction]
#[pyo3(text_signature = "(img: numpy.ndarray) -> List[ImageSegment]",
            name="find_stars")]
pub fn py_find_stars(img: &Bound<'_, PyAny>) -> PyResult<Vec<PyImageSegment>> {

    if img.is_instance_of::<np::PyArray2<u16>>() {
        let img: np::PyReadonlyArray2<u16> = img.extract()?;
        let arr = img.as_array();
        let dims = arr.shape();
        Ok(find_stars_py(img.as_slice().unwrap(), dims[0], dims[1]))
    }
    else if img.is_instance_of::<np::PyArray2<u8>>() {
        let img: np::PyReadonlyArray2<u8> = img.extract()?;
        let arr = img.as_array();
        let dims = arr.shape();
        Ok(find_stars_py(img.as_slice().unwrap(), dims[0], dims[1]))
    }
    else {
        Err(pyo3::exceptions::PyTypeError::new_err("Image must be a 2D numpy array"))
    }    
}