use pyo3::prelude::*;
use numpy as np;

use crate::starfinder::{Segment, find_stars, FindStarsOptions};

#[pyclass(name="FindStarsOptions")]
#[derive(Clone, Debug)]
pub struct PyFindStarsOptions {
    inner: FindStarsOptions
}

#[pymethods]
impl PyFindStarsOptions {
    #[new]
    fn new() -> PyFindStarsOptions {
        PyFindStarsOptions {
            inner: FindStarsOptions::defaults(),
        }
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }

    #[getter]
    fn get_minsize(&self) -> usize {
        self.inner.minsize
    }

    #[setter(minsize)]
    fn set_minsize(&mut self, s: usize) {
        self.inner.minsize = s
    }

    #[getter]
    fn get_threshold(&self) -> f64 {
        self.inner.threshold
    }

    #[setter(threshold)]
    fn set_threshold(&mut self, t: f64) {
        self.inner.threshold = t
    }
}


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
    fn get_indices(&self) -> Vec<(usize, usize)> {
        self.inner.indices.clone()
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

fn find_stars_py<T> (pix: &[T], rows: usize, cols: usize, options: Option<FindStarsOptions>) -> Vec<PyImageSegment>
    where T: Sized + Clone + Copy + Into<i64>
            + Into<f64> + Into<i32>
            + std::cmp::Ord
            + std::ops::Add<Output = T>
{
    let segments = find_stars(&pix, rows, cols, options);
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
            name="find_stars",
            signature=(img, options=None)
        )]
pub fn py_find_stars(img: &Bound<'_, PyAny>, options: Option<PyFindStarsOptions>) -> PyResult<Vec<PyImageSegment>> {

    let options: Option<FindStarsOptions> = match options {
        None => None,
        Some(s) => Some(s.inner),
    };

    if img.is_instance_of::<np::PyArray2<u16>>() {
        let img: np::PyReadonlyArray2<u16> = img.extract()?;
        let arr = img.as_array();
        let dims = arr.shape();
        Ok(find_stars_py(img.as_slice().unwrap(), dims[0], dims[1], options))
    }
    else if img.is_instance_of::<np::PyArray2<u8>>() {
        let img: np::PyReadonlyArray2<u8> = img.extract()?;
        let arr = img.as_array();
        let dims = arr.shape();
        Ok(find_stars_py(img.as_slice().unwrap(), dims[0], dims[1], options))
    }
    else {
        Err(pyo3::exceptions::PyTypeError::new_err("Image must be a 2D numpy array"))
    }    
}