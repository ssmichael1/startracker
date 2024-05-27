use thiserror::Error;

use crate::cameraframe::{CameraFrameMono16, CameraFrame, FrameTime, PixelOrder};

//use std::io;
use std::io::prelude::*;

#[derive(Error, Debug)]
pub enum SERFileError {
    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),
    #[error("Invalid Header")]
    InvalidHeader,
    #[error("Invalid Color ID: {0}")]
    InvalidColorID(u32),
    #[error("Invalid Endian in Header: {0}")]
    InvalidEndian(u32),
}

#[derive(Clone, Debug)]
pub enum Endian {
    Little,
    Big,
}

impl std::convert::TryFrom<u32> for Endian {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 =>  Ok(Endian::Little),
            1 => Ok(Endian::Big),
            v => Err(Box::new(SERFileError::InvalidEndian(v)))
        }
    }
}

#[derive(Debug, Clone)]
pub enum ColorID {
    Mono = 0,
    BayerRGGB=8,
    BayerGRBG=9,
    BayerBGRG=10,
    BayerBGGR=11,
    BayerCYYM=16,
    BayerYCMY=17,
    BayerYMCY=18,
    BayerMYYC=19,
    RGB=100,
    BGR=101,
}

impl std::fmt::Display for ColorID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ColorID::Mono => write!(f, "Mono"),
            ColorID::BayerRGGB => write!(f, "BayerRGGB"),
            ColorID::BayerGRBG => write!(f, "BayerGRBG"),
            ColorID::BayerBGRG => write!(f, "BayerBGRG"),
            ColorID::BayerBGGR => write!(f, "BayerBGGR"),
            ColorID::BayerCYYM => write!(f, "BayerCYYM"),
            ColorID::BayerYCMY => write!(f, "BayerYCMY"),
            ColorID::BayerYMCY => write!(f, "BayerYMCY"),
            ColorID::BayerMYYC => write!(f, "BayerMYYC"),
            ColorID::RGB => write!(f, "RGB"),
            ColorID::BGR => write!(f, "BGR"),
        }
    }
}

impl std::convert::TryFrom<u32> for ColorID {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(ColorID::Mono),
            8 => Ok(ColorID::BayerRGGB),
            9 => Ok(ColorID::BayerGRBG),
            10 => Ok(ColorID::BayerBGRG),
            11 => Ok(ColorID::BayerBGGR),
            16 => Ok(ColorID::BayerCYYM),
            17 => Ok(ColorID::BayerYCMY),
            18 => Ok(ColorID::BayerYMCY),
            19 => Ok(ColorID::BayerMYYC),
            100 => Ok(ColorID::RGB),
            101 => Ok(ColorID::BGR),
            v => Err(Box::new(SERFileError::InvalidColorID(v)))
        }
    }
}


#[derive(Debug, Clone)]
pub struct SERFile {
    pub color_id: ColorID,
    pub endian: Endian,
    pub width: usize,
    pub height: usize,
    pub frame_count: usize,
    pub bit_depth: usize,
    pub fname: String,
    pub instrument: String,
    pub observer: String,
    pub telescope: String,
    pub frames: Vec<CameraFrame>,
}

impl SERFile {
    pub fn new(fname: &String) -> 
    Result<SERFile, Box<dyn std::error::Error + Send + Sync>>
    {
        use std::path::PathBuf;

        let path = PathBuf::from(fname.clone());
        if !path.is_file() {
            return Err(Box::new(SERFileError::FileDoesNotExist(fname.clone())));
        }

        const HEADER_LEN: usize = 178;
        const HEADER_STRING: &str = "LUCAM-RECORDER";

        let mut fs = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(Box::new(e))
        };
        let mut header: [u8; HEADER_LEN] = [0u8; HEADER_LEN];
        fs.read_exact(&mut header)?;
        

        if std::str::from_utf8(&header[0..HEADER_STRING.len()])? != HEADER_STRING {
            return Err(Box::new(SERFileError::InvalidHeader))
        }
        //let _idd = u32::from_le_bytes(header[14..18].try_into()?);
        let color_id: ColorID = u32::from_le_bytes(header[18..22].try_into()?).try_into()?;
        let endian: Endian = u32::from_le_bytes(header[22..26].try_into()?).try_into()?;
        let width = u32::from_le_bytes(header[26..30].try_into()?) as usize;
        let height = u32::from_le_bytes(header[30..34].try_into()?) as usize;
        let bit_depth = u32::from_le_bytes(header[34..38].try_into()?) as usize;
        let frame_count = u32::from_le_bytes(header[38..42].try_into()?) as usize;
        let observer = std::str::from_utf8(&header[42..82])?;
        let instrument = std::str::from_utf8(&header[82..122])?;
        let telescope = std::str::from_utf8(&header[122..152])?;

        let pixel_layers: usize = match color_id {
            ColorID::RGB => 3,
            ColorID::BGR => 3,
            _ => 1,
        };
        let byte_depth = (bit_depth + 7) / 8;
        let frame_bytes = width * height * byte_depth * pixel_layers;
        
        // Read in the entire file
        let mut frames_buf = vec![0u8; frame_bytes * frame_count];
        fs.read_exact(&mut frames_buf)?;
        


        let mut time_buf = vec![0u8;frame_count * 8];
        fs.read_exact(&mut time_buf)?;
        let epoch = chrono::NaiveDate::from_ymd_opt(1, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0).unwrap();
        let frame_times = time_buf.as_slice().chunks_exact(8).map(|x| {
            let val = u64::from_le_bytes(x.try_into().unwrap());
            let usecs  = val / 10;
            let nsecs = (val % 100) * 10;
            let time: chrono::NaiveDateTime = epoch
                + chrono::Duration::microseconds(usecs as i64)
                + chrono::Duration::nanoseconds(nsecs as i64);
            time.and_utc()
        }).collect::<Vec<FrameTime>>();

        let frames: Vec<CameraFrame> = frames_buf
            .chunks_exact(frame_bytes)
            .enumerate()
            .map(|(idx, x)| -> CameraFrame {
            CameraFrame::Mono16(
                CameraFrameMono16::new(width, height, bit_depth, PixelOrder::ColMajor,
                    &frame_times[idx].into(),
                    unsafe {
                        std::slice::from_raw_parts(x.as_ptr() as *const u16, x.len() / 2)
                    }
                )
            )
        }).collect();
    
        Ok(SERFile {
            color_id: color_id,
            endian: endian,
            width: width as usize,
            height: height as usize,
            bit_depth: bit_depth as usize,
            frame_count: frame_count as usize,
            fname: String::from(fname),
            instrument: String::from(instrument.trim()),
            observer: String::from(observer.trim()),
            telescope: String::from(telescope.trim()),
            frames: frames,
        })        


    }
}

impl std::fmt::Display for SERFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SERFile: {}\n", self.fname)?;
        write!(f, "   Instrument: {}\n", self.instrument)?;
        write!(f, "     Observer: {}\n", self.observer)?;
        write!(f, "    Telescope: {}\n", self.telescope)?;
        write!(f, "     Color ID: {}\n", self.color_id)?;
        write!(f, "       Endian: {:?}\n", self.endian)?;
        write!(f, "        Width: {}\n", self.width)?;
        write!(f, "       Height: {}\n", self.height)?;
        write!(f, "    Bit Depth: {}\n", self.bit_depth)?;
        write!(f, "  Frame Count: {}\n", self.frame_count)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cameraframe::{CameraFrameData, CameraFrameTraits};
    use crate::find_stars;

    #[test]
    fn load_ser() {
        let f = SERFile::new(
            &String::from("../stardata/2024-05-07-fixed_25mm__000007__21-33-42__data.ser")).unwrap();
        println!("f = {}", f);
        match f.frames[0] {
            CameraFrame::Mono16(ref frame) => {
                println!("Frame: {}", frame);
                let data = frame.as_slice();
                let stars = find_stars(data, frame.rows(), frame.cols());
                println!("Number of stars: {}", stars.len());
            },
            
            _ => panic!("Expected Mono16 frame")
        };
    }
}