use std::ops::{Index, IndexMut};
use chrono::prelude::{DateTime, Utc};
use thiserror::Error;
use rand::{distributions::Uniform, Rng};

pub type FrameTime = DateTime<Utc>;
pub type FrameResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;


#[derive(Error, Debug)]
pub enum CameraFrameError {
    #[error("Index Out Of Bounds {0}")]
    IndexOutOfBounds(usize),
}


#[derive(Clone, Debug)]
pub enum PixelFormat {
    MONO,
    BayerRGGB,
    BayerGRBG,
    BayerBGRG,
    BayerCYYM,
    BayerYCMY,
    BayerMYYC,
    RGB,    
}

impl std::fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PixelFormat::MONO => write!(f, "MONO"),
            PixelFormat::BayerRGGB => write!(f, "BayerRGGB"),
            PixelFormat::BayerGRBG => write!(f, "BayerGRBG"),
            PixelFormat::BayerBGRG => write!(f, "BayerBGRG"),
            PixelFormat::BayerCYYM => write!(f, "BayerCYYM"),
            PixelFormat::BayerYCMY => write!(f, "BayerYCMY"),
            PixelFormat::BayerMYYC => write!(f, "BayerMYYC"),
            PixelFormat::RGB => write!(f, "RGB"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PixelOrder {
    RowMajor,
    ColMajor,
}

impl std::fmt::Display for PixelOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PixelOrder::RowMajor => write!(f, "RowMajor"),
            PixelOrder::ColMajor => write!(f, "ColMajor"),
        }
    }
}
pub trait CameraFrameTraits {
    fn rows(&self) -> usize;
    fn cols(&self) -> usize;
    fn bit_depth(&self) -> usize;
    fn pixel_format(&self) -> PixelFormat;
    fn pixel_order(&self) -> &PixelOrder;
    fn time(&self) -> &FrameTime;
}

pub trait CameraFrameData<T>  where T: Sized + Clone
{
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut[T];
}

impl std::fmt::Display for dyn CameraFrameTraits
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Camera Frame\n")?;
        write!(f, "  Pixel Format: {}\n", self.pixel_format())?;
        write!(f, "  Pixel Order: {}\n", self.pixel_order())?;
        write!(f, "  Rows: {}\n", self.rows())?;
        write!(f, "  Cols: {}\n", self.cols())?;
        write!(f, "  Bit Depth: {}\n", self.bit_depth())?;
        write!(f, "  Time: {}\n", self.time().to_rfc3339())?;
        Ok(())
    }
}

pub trait Get<Idx, T> 
where Idx: Sized, T: Sized + Clone, 
{
    fn at(&self, index: Idx) -> FrameResult<&T>;
}

impl<C,T> Get<usize, T> for  C
where 
    C: CameraFrameData<T> + CameraFrameTraits,
    T: Sized + Clone
{
    fn at(&self, index: usize) -> FrameResult<&T> {
        if index > self.rows() * self.cols() {
            return Err(Box::new(CameraFrameError::IndexOutOfBounds(index)));
        }
       Ok(&self.as_slice()[index])
    }
}

impl<C,T> Get<(usize,usize), T> for C
where 
    C: CameraFrameTraits + CameraFrameData<T>,
    T: Sized + Clone

{
    fn at(&self, (r,c): (usize, usize)) -> FrameResult<&T> {
        let index = match self.pixel_order() {
            PixelOrder::ColMajor => c*self.rows() + r,
            PixelOrder::RowMajor => r*self.cols() + c,
        };

        if r >= self.rows() || c >= self.cols() {
            return Err(Box::new(CameraFrameError::IndexOutOfBounds(index)));
        }
        Ok(&self.as_slice()[index])
    }
}

#[derive(Debug, Clone)]
pub struct CameraFrameMono<T>
where T: Sized + Clone
{
    rows: usize,
    cols: usize,
    bit_depth: usize,
    rawdata: Vec<T>,
    pixel_order: PixelOrder,
    time: FrameTime,
}


impl<T> CameraFrameTraits for CameraFrameMono<T>
where T: Sized + Clone
{
    fn rows(&self) -> usize { self.rows }
    fn cols(&self) -> usize { self.cols }
    fn bit_depth(&self) -> usize {self.bit_depth}
    fn pixel_format(&self) -> PixelFormat { PixelFormat::MONO }
    fn pixel_order(&self) -> &PixelOrder { &self.pixel_order }
    fn time(&self) -> &FrameTime {
        &self.time
    }
}

impl<T>  CameraFrameData<T> for CameraFrameMono<T>
where T: Sized + Clone
{
    fn as_slice(&self) -> &[T] {
        self.rawdata.as_slice()
    }
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.rawdata.as_mut_slice()
    }
}

impl<T> Index<usize> for CameraFrameMono<T>
where T: Sized + Clone
{
    type Output = T;

    fn index<'a>(&'a self, i: usize) -> &'a T {
        &self.rawdata[i]
    }
}

impl<T> IndexMut<usize> for CameraFrameMono<T>
where T: Sized + Clone
{
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut T {
        &mut self.rawdata[i]
    }
}

impl<T> Index<(usize,usize)> for CameraFrameMono<T>
where T: Sized + Clone
{
    type Output = T;
    fn index<'a>(&'a self, (r,c): (usize, usize)) -> &'a T {
        match self.pixel_order {
            PixelOrder::ColMajor => &self.rawdata[c*self.rows + r],
            PixelOrder::RowMajor => &self.rawdata[r*self.cols + c]
        }
    }
}

impl<T> IndexMut<(usize, usize)> for CameraFrameMono<T>
where T: Sized + Clone
{
    fn index_mut<'a>(&'a mut self, (r, c): (usize, usize)) -> &'a mut T {
        match self.pixel_order {
            PixelOrder::ColMajor => &mut self.rawdata[c*self.rows + r],
            PixelOrder::RowMajor => &mut self.rawdata[r*self.cols + c],
        }
    }
}

impl <T> CameraFrameMono<T> 
where T: num::Integer + Sized + Clone + rand::distributions::uniform::SampleUniform + num::Zero
{
    pub fn zeros(rows: usize, cols: usize, bit_depth: usize) -> CameraFrameMono<T> {
        CameraFrameMono::<T> {
            rows: rows,
            cols: cols,
            bit_depth: bit_depth,
            pixel_order: PixelOrder::ColMajor,
            rawdata: vec![num::zero() ; rows*cols],
            time: std::time::SystemTime::now().into()
        }
    }
    pub fn new(rows: usize, cols: usize, bit_depth: usize, pixel_order: PixelOrder,
        time: &FrameTime, data: &[T]) -> CameraFrameMono<T> {
        CameraFrameMono::<T> {
            rows: rows,
            cols: cols,
            bit_depth: bit_depth,
            pixel_order: pixel_order,
            rawdata: data.to_vec(),
            time: time.clone(),
        }
    }
}

impl CameraFrameMono8
{
    pub fn rand(rows: usize, cols: usize) -> CameraFrameMono8 {
        let mut rng = rand::thread_rng();

        let range: Uniform<u16> = Uniform::new(0, 256);
        CameraFrameMono8 {
            rows: rows,
            cols: cols,
            bit_depth: 8,
            pixel_order: PixelOrder::ColMajor,
            rawdata: (0..(rows*cols)).map(|_| rng.sample(&range) as u8).collect(),
            time: std::time::SystemTime::now().into(),
        }
    }
}

impl CameraFrameMono16
{
    pub fn rand(rows: usize, cols: usize) -> CameraFrameMono16 {
        let mut rng = rand::thread_rng();

        let range: Uniform<u16> = Uniform::new(0, 256);
        CameraFrameMono16 {
            rows: rows,
            cols: cols,
            bit_depth: 8,
            pixel_order: PixelOrder::ColMajor,
            rawdata: (0..(rows*cols)).map(|_| rng.sample(&range)).collect(),
            time: std::time::SystemTime::now().into(),
        }
    }
}

impl<T> std::fmt::Display for CameraFrameMono<T>
where T: Sized + Clone
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) ->std::fmt::Result {
        write!(f, "Monochromatic Camera Frame\n")?;
        write!(f, "  Pixel Format: {} X {}\n", self.rows, self.cols)?;
        write!(f, "   Pixel Order: {}\n", self.pixel_order)?;
        write!(f, "     Bit Depth: {}\n", self.bit_depth)?;
        write!(f, "    Frame Time: {}\n", self.time.to_rfc3339())?;
        Ok(())

    }
}

pub type CameraFrameMono8 = CameraFrameMono<u8>;
pub type CameraFrameMono16 = CameraFrameMono<u16>;

#[derive(Debug, Clone)]
pub enum CameraFrame {
    Mono8(CameraFrameMono8),
    Mono16(CameraFrameMono16),
}

impl CameraFrameTraits for CameraFrame {
    fn rows(&self) -> usize {
        match self {
            CameraFrame::Mono8(x) => x.rows(),
            CameraFrame::Mono16(x) => x.rows(),
        }
    }
    fn cols(&self) -> usize {
        match self {
            CameraFrame::Mono8(x) => x.cols(),
            CameraFrame::Mono16(x) => x.cols(),
        }
    }
    fn bit_depth(&self) -> usize {
        match self {
            CameraFrame::Mono8(x) => x.bit_depth(),
            CameraFrame::Mono16(x) => x.bit_depth(),
        }
    }
    fn pixel_format(&self) -> PixelFormat {
        match self {
            CameraFrame::Mono8(x) => x.pixel_format(),
            CameraFrame::Mono16(x) => x.pixel_format(),
        }
    }
    fn pixel_order(&self) -> &PixelOrder {
        match self {
            CameraFrame::Mono8(x) => x.pixel_order(),
            CameraFrame::Mono16(x) => x.pixel_order(),
        }
    }
    fn time(&self) -> &FrameTime {
        match self {
            CameraFrame::Mono8(x) => x.time(),
            CameraFrame::Mono16(x) => x.time(),
        }
    }
}

impl std::fmt::Display for CameraFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CameraFrame::Mono8(x) => write!(f, "{}", x),
            CameraFrame::Mono16(x) => write!(f, "{}", x),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn zeroframe() {
        let mut z = CameraFrameMono8::rand(64, 64);
        z[(1,0)] = 3;
        z[(0,1)] = 4;
        println!("{}", z);
        println!("{:?}", z.at((3,53)));
    }
}
