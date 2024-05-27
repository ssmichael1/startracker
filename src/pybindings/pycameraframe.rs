use pyo3::prelude::*;
use pyo3::types::PyDateTime;
use numpy as np;

use crate::cameraframe::{CameraFrame, CameraFrameTraits, CameraFrameData, PixelFormat, PixelOrder};

#[pyclass(name="PixelOrder")]
#[derive(Clone, Debug)]
pub enum PyPixelOrder {
    ColMajor = PixelOrder::ColMajor as isize,
    RowMajor = PixelOrder::RowMajor as isize,
}

impl Into<PixelOrder> for PyPixelOrder {
    fn into(self) -> PixelOrder {
        match self {
            PyPixelOrder::ColMajor => PixelOrder::ColMajor,
            PyPixelOrder::RowMajor => PixelOrder::RowMajor,
        }
    }
}

impl Into<PyPixelOrder> for PixelOrder {
    fn into(self) -> PyPixelOrder {
        match self {
            PixelOrder::ColMajor => PyPixelOrder::ColMajor,
            PixelOrder::RowMajor => PyPixelOrder::RowMajor,
        }
    }
}

#[pyclass(name="PixelFormat")]
#[derive(Clone, Debug)]
pub enum PyPixelFormat {
    Mono = PixelFormat::MONO as isize,
    RGB = PixelFormat::RGB as isize,
    BayerRGGB = PixelFormat::BayerRGGB as isize,
    BayerGRBG = PixelFormat::BayerGRBG as isize,
    BayerBGRG = PixelFormat::BayerBGRG as isize,
    BayerCYYM = PixelFormat::BayerCYYM as isize,
    BayerYCMY = PixelFormat::BayerYCMY as isize,
    BayerMYYC = PixelFormat::BayerMYYC as isize,
}

impl Into<PixelFormat> for PyPixelFormat {
    fn into(self) -> PixelFormat {
        match self {
            PyPixelFormat::Mono => PixelFormat::MONO,
            PyPixelFormat::RGB => PixelFormat::RGB,
            PyPixelFormat::BayerRGGB => PixelFormat::BayerRGGB,
            PyPixelFormat::BayerGRBG => PixelFormat::BayerGRBG,
            PyPixelFormat::BayerBGRG => PixelFormat::BayerBGRG,
            PyPixelFormat::BayerCYYM => PixelFormat::BayerCYYM,
            PyPixelFormat::BayerYCMY => PixelFormat::BayerYCMY,
            PyPixelFormat::BayerMYYC => PixelFormat::BayerMYYC,
        }
    }
}

impl Into<PyPixelFormat> for PixelFormat {
    fn into(self) -> PyPixelFormat {
        match self {
            PixelFormat::MONO => PyPixelFormat::Mono,
            PixelFormat::RGB => PyPixelFormat::RGB,
            PixelFormat::BayerRGGB => PyPixelFormat::BayerRGGB,
            PixelFormat::BayerGRBG => PyPixelFormat::BayerGRBG,
            PixelFormat::BayerBGRG => PyPixelFormat::BayerBGRG,
            PixelFormat::BayerCYYM => PyPixelFormat::BayerCYYM,
            PixelFormat::BayerYCMY => PyPixelFormat::BayerYCMY,
            PixelFormat::BayerMYYC => PyPixelFormat::BayerMYYC,
        }
    }
}   



#[pyclass(name="CameraFrame")]
#[derive(Clone, Debug)]
pub struct PyCameraFrame {
    pub inner: CameraFrame,
}

#[pymethods]
impl PyCameraFrame {
 
    #[getter]
    fn get_width(&self) -> usize {
        self.inner.cols()
    }

    #[getter]
    fn get_height(&self) -> usize {
        self.inner.rows()
    }

    #[getter]
    fn get_rows(&self) -> usize {
        self.inner.rows()
    }
 
    #[getter]
    fn get_cols(&self) -> usize {
        self.inner.cols()
    }

    #[getter]
    fn get_format(&self) -> (usize, usize) {
        (self.inner.rows(), self.inner.cols())
    }

    #[getter]
    fn get_bit_depth(&self) -> usize {
        self.inner.bit_depth()
    }

    #[getter]
    fn get_pixel_format(&self) -> PyPixelFormat {
        self.inner.pixel_format().into()
    }

    #[getter]
    fn get_pixel_order(&self) -> PyPixelOrder {
        self.inner.pixel_order().clone().into()
    }

    #[getter]
    fn pixels(&self) -> PyResult<PyObject> {
            match self.inner {
                CameraFrame::Mono8(ref _array) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Not implemented"));
                },
                CameraFrame::Mono16(ref array) => {
                    pyo3::Python::with_gil(|py| -> PyResult<PyObject> {
                        Ok(
                            np::PyArray1::from_slice_bound(py, array.as_slice())
                            .as_gil_ref()
                            .reshape([self.inner.cols() as usize, self.inner.rows() as usize])?
                            .to_object(py)
                        )
                    })
                },
            } 
    }

    #[getter]
    fn get_time(&self) -> PyResult<PyObject> {

        let time = self.inner.time();
        let timestamp = time.timestamp();
        pyo3::Python::with_gil(|py| {
            Ok(PyDateTime::from_timestamp_bound(py, timestamp as f64, None)?.into_py(py))
        })
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }
}