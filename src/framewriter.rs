//! Save camera frame to .png file
//! 
//! 





use crate::cameraframe::{CameraFrame, CameraFrameData, CameraFrameTraits};
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FrameWriterError {
    #[error("Cannot write frame with this format")]
    CannotWriteFormat
}

pub fn save_frame_to_png(frame: &CameraFrame, filename: &str) -> Result<(), Box<dyn std::error::Error>> {

    let path = Path::new(filename).with_extension("png");
    let file = File::create(path)?;
    let ref mut w = BufWriter::new(file);

    match frame {
        CameraFrame::Mono8(_frame) => {
            return Err(FrameWriterError::CannotWriteFormat.into());
        },
        CameraFrame::Mono16(frame) => {
            let mut encoder = 
                png::Encoder::new(w, frame.rows() as u32, frame.cols() as u32);
            encoder.set_color(png::ColorType::Grayscale);
            encoder.set_depth(png::BitDepth::Sixteen);
            let mut writer = encoder.write_header()?;
            writer.write_image_data(unsafe {
                std::slice::from_raw_parts(frame.as_slice().as_ptr() as *const u8, frame.as_slice().len() * 2)
            })?;
        },
    }
   
    Ok(())
}