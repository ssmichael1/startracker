

#[derive(Debug, Clone)]
pub struct FindStarsOptions {
    pub threshold: f64,
    pub minsize: usize,
}

impl FindStarsOptions {
    pub fn defaults() -> FindStarsOptions {
        FindStarsOptions {
            threshold: 2.5,
            minsize: 2,
        }
    }
}

impl std::fmt::Display for FindStarsOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Options:\n")?;
        write!(f, "               threshold: {:.2}\n", self.threshold)?;
        write!(f, "    Minimum segment size: {} pixels", self.minsize)
    }    
}


#[derive(Debug, Clone)]
pub struct Segment {
    pub indices: Vec<(usize, usize)>,
    pub centroid: (f64, f64),
    pub mass: f64,
}

impl Segment {
    pub fn new() -> Self {
        Segment {
            indices: Vec::<(usize, usize)>::new(),
            centroid: (0.0, 0.0),
            mass: 0.0,
        }
    }
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Segment: indices: {:?}, centroid: {:?}, mass: {}", 
            self.indices, self.centroid, self.mass)
    }
}

/// Low-level function to find the stars and report out 
/// locations and brightnesses in the image
pub fn find_stars<T> (pix: &[T], rows: usize, cols: usize, options: Option<FindStarsOptions>) -> Vec<Segment>
    where T: Sized + Clone + Copy + Into<i64>
            + Into<f64> + Into<i32>
            + std::cmp::Ord
            + std::ops::Add<Output = T>
{

    let options = match options {
        None => FindStarsOptions::defaults(),
        Some(o) => o,
    };

    let mut min: i32 = std::i32::MAX;
    let mut max: i32 = std::i32::MIN;
    let mut sum: i64 = 0;
    let mut sumsq: i64 = 0;
    
    // Iterate over the pixels and calculate the min, max, sum, and sum of squares
    pix.iter().for_each(|val| {
        let v32: i32 = val.clone().into();
        let v64: i64 = val.clone().into();
        sum += v64;
        sumsq += v64 * v64;
        if v32 < min {
            min = v32;
        }
        if v32 > max {
            max = v32;
        }
    });
    let count = pix.len();

    let mean = sum as f64 / count as f64;
    let std_dev = ((sumsq as f64 - count as f64 * mean * mean) / (count as f64 - 1.0)).sqrt();
    let thresh: i32 = (mean + options.threshold*std_dev) as i32;

    // Create a pixel mask to mark the pixel segments
    let mut mask: Vec<i32> = vec![-1; pix.len()];
    let mut segments = Vec::<Segment>::new();
    let mut segment_count = 0;

    // Iterate over the pixels and create the mask
    pix.iter().enumerate().for_each(|(idx, v)| {
        let v32: i32 = v.clone().into();
        if v32 > thresh {
            if mask[idx] == -1 {
                // We found a new segment
                let mut new_segment = Segment::new();
                new_segment.indices.push((idx / cols, idx % cols));
                mask[idx] = segment_count;
                segment_count += 1;
                segments.push(new_segment);
            }
            
            // Check connectivitity to the right
            if (idx%cols) +1 < cols {
                let right = idx + 1;
                let vright: i32 = pix[right].clone().into();
                if mask[right] == -1 && vright > thresh {
                    mask[right] = mask[idx];
                    segments[mask[idx] as usize].indices.push((right / cols, right % cols));
                }
            }

            // Is there a row below?
            if (idx / cols) < (rows-1) {
                let current_col = idx%cols;

                // Check connectivity to the pixel below left
                if current_col > 0  {
                    let belowleft = idx + cols - 1;
                    let vbelowleft: i32 = pix[belowleft].clone().into();
                    if mask[belowleft] == -1 && vbelowleft > thresh {
                        mask[belowleft] = mask[idx];
                        segments[mask[idx] as usize].indices.push((belowleft / cols, belowleft % cols));
                    }
                }

                // Check connectivity to the pixel below
                let below = idx + cols;
                let vbelow: i32 = pix[below].clone().into();
                if mask[below] == -1 && vbelow > thresh {
                    mask[below] = mask[idx];
                    segments[mask[idx] as usize].indices.push((below / cols, below % cols));
                }
                

                // Check connectivity to the pixel below right
                if current_col + 1 < cols {
                    let belowright = idx + cols + 1;
                    let vbelowright: i32 = pix[belowright].clone().into();
                    if mask[belowright] == -1 && vbelowright > thresh {
                        mask[belowright] = mask[idx];
                        segments[mask[idx] as usize].indices.push((belowright / cols, belowright % cols));
                    }
                }
            }
        }
    });

    // Remove elements less than the minimum size in pixels
    segments.retain(|x| x.indices.len() >= options.minsize);
    

    // Calculate the centroid and mass of each segment
    segments.iter_mut().for_each(|segment| {
        let mut sumx = 0.0;
        let mut sumy = 0.0;
        let mut summass = 0.0;

        // Find the center row and column
        let (srow, scol) = segment.indices.iter().fold((0, 0), |acc, (r, c)| {
            (acc.0 + r, acc.1 + c)
        });
        let crow = srow as f64 / segment.indices.len() as f64;
        let ccol = scol as f64 / segment.indices.len() as f64;

        // Centroid about the center row and column
        // this is important as it reduces noise in the centroid calculation
        segment.indices.iter().for_each(|(row, col)| {
            let val: f64 = pix[row * cols + col].clone().into();
            sumx += val * (col.clone() as f64 - ccol);
            sumy += val * (crow.clone() as f64 - crow);
            summass += val;
        });
        segment.centroid = (sumx / summass + ccol as f64, sumy / summass + crow as f64);
        segment.mass = summass;
    });
    // Sort by mass (Signal)
    segments.sort_by(|a,b| {
        a.mass.partial_cmp(&b.mass).unwrap()
    });


    segments

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic() {
        let rows: i32 = 64;
        let cols: i32 = 64;
        let sigma: f64 = 1.0;

        let centroids: Vec<(f64, f64)> = vec![(4.0, 3.0), (-2.0, 3.2), (-3.4, -6.5)];
        //let centroids: Vec<(f64, f64)> = vec![(0.0, 0.0)];
        let gauss: Vec<u16> = (0..(rows*cols) as i32).into_iter().map(|idx| {
            let x = idx % cols - cols/2;
            let y = idx / cols - rows/2;
            let mut val = 0.0;
            for (cx, cy) in centroids.iter() {
                let dx = x as f64 - cx;
                let dy = y as f64 - cy;
                val += 256.0 * (-dx*dx/2.0/sigma/sigma - dy*dy/2.0/sigma/sigma).exp();
            }
            val as u16
        }).collect();
    
        let mut options = FindStarsOptions::defaults();
        options.threshold = 503.0;

        let segments = super::find_stars(&gauss, rows as usize, cols as usize, Some(options));
        println!("Found {} segments", segments.len());
        for segment in segments.iter() {
            println!("Segment: {}", segment);
        }
    }
}
