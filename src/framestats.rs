use crate::cameraframe::{CameraFrameMono, CameraFrameData};

#[derive(Debug, Clone)]
pub struct FrameStats {
    pub min: i64,
    pub max: i64,
    pub mean: f64,
    pub median: i64,
    pub std_dev: f64,
}


impl FrameStats
{
    pub fn new<T>(frame: &CameraFrameMono<T>) -> Self 
        where T: Sized + Clone + Copy + Into<i64> + Into<f64>
                + std::cmp::Ord
                + std::ops::Add<Output = Self>
    {
        let mut min: i64 = std::i64::MAX;
        let mut max: i64 = std::i64::MIN;
        let mut sum: i64 = 0;
        let mut sumsq: i64 = 0;
        
        let raw = frame.as_slice();
        raw.iter().for_each(|val| {
            let val: i64 = val.clone().into();
            sum += val;
            sumsq += val*val;
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        });
        let count = raw.len();
        let mut values: Vec<i64> = raw.iter().map(|val| val.clone().into()).collect();
        values.sort_unstable();
        let median = values[count/2].clone().into();
        let mean = sum as f64 / count as f64;
        let std_dev = ((sumsq as f64 - count as f64 * mean * mean) / (count as f64 - 1.0)).sqrt();

        FrameStats {
            min,
            max,
            mean,
            median,
            std_dev,
        }
    }
}