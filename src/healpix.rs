//!
//! HEALPIX algorithm transcribed from C
//!
//! The original C code is available at:
//! https://healpix.sourceforge.io/
//!
//! The original C code is licensed under the GNU General Public License v2.0
//!
//! Currently only the `pix2ang_ring` and `ang2pix_ring` functions are implemented.
//!
//! The `pix2ang_ring` function converts a pixel number to a pair of angles (theta, phi) in radians
//! where theta is the polar angle and phi is the azimuthal angle.
//!
//! The `ang2pix_ring` function converts a pair of angles (theta, phi) in radians to a pixel number.
//!
//! Polar angle is defined as the angle from the z-axis (angle from "North"),
//! and azimuthal angle is defined as the angle from the x-axis (angle from "East").
//! The angles are in the range [0, pi] and [0, 2 * pi) respectively.
//!
use std::f64::consts::PI;

use num::integer::Roots;

fn pix2ang_ring_z_phi(nside: u32, pix: u32) -> (f64, f64) {
    let pix = pix as i32;
    let nside = nside as i32;
    let ncap = nside * (nside - 1) * 2;
    let npix = 12 * nside * nside;
    let fact2 = 4.0 / npix as f64;
    if pix < ncap {
        // North polar ca
        let iring: i32 = (1 + (1 + 2 * pix).sqrt()) >> 1;
        let iphi = pix + 1 - 2 * iring * (iring - 1);
        let z = 1.0 - (iring * iring) as f64 * fact2;
        let phi = (iphi as f64 - 0.5) * PI / (2.0 * iring as f64);
        (z, phi)
    } else if pix < (npix - ncap) {
        // Equatorial region
        let fact1 = (nside << 1) as f64 * fact2;
        let ip = pix - ncap;
        let iring = ip / (4 * nside) + nside;
        let iphi = ip % (4 * nside) + 1;
        let fodd = match (iring + nside) & 1 {
            0 => 0.5,
            _ => 1.0,
        };
        let nl2 = 2 * nside;
        let z = (nl2 - iring) as f64 * fact1;
        let phi = (iphi as f64 - fodd) * PI / nl2 as f64;
        (z, phi)
    } else {
        // South polar cap
        let ip = npix - pix;
        let iring = (1 + (2 * ip - 1).sqrt()) >> 1;
        let iphi = 4 * iring + 1 - (ip - 2 * iring * (iring - 1));
        let z = -1.0 + (iring * iring) as f64 * fact2;
        let phi = (iphi as f64 - 0.5) * std::f64::consts::PI / (2.0 * iring as f64);
        (z, phi)
    }
}

fn ang2pix_ring_z_phi(nside: u32, z: f64, phi: f64) -> u32 {
    let nside = nside as i32;
    let za = z.abs();
    let tt = (phi % (2.0 * PI)) * 2.0 / PI;
    if za < (2.0 / 3.0) {
        // Equatorial region
        let temp1 = nside as f64 * (0.5 + tt);
        let temp2 = nside as f64 * z * 0.75;
        let jp = (temp1 - temp2) as i32;
        let jm = (temp1 + temp2) as i32;
        let ir = nside + 1 + jp - jm;
        let kshift = 1 - (ir & 1);
        let mut ip = (jp + jm - nside + kshift + 1) / 2;
        ip = ip % (4 * nside);
        (nside * (nside - 1) * 2 + (ir - 1) * 4 * nside + ip) as u32
    } else {
        // North and South polar caps
        let tp = tt.fract();
        let tmp = nside as f64 * (3.0 * (1.0 - za)).sqrt();
        let jp = (tp * tmp) as i32;
        let jm = ((1.0 - tp) * tmp) as i32;
        let ir = jp + jm + 1;
        let mut ip = (tt * ir as f64) as i32;
        ip = ip % (4 * ir);
        if z >= 0.0 {
            (2 * ir * (ir - 1) + ip) as u32
        } else {
            (12 * nside * nside - 2 * ir * (ir + 1) + ip) as u32
        }
    }
}

///
/// Convert a pixel number to a pair of angles (theta, phi) in radians
/// where theta is the polar angle and phi is the azimuthal angle.
/// The pixel number is in the range [0, 12 * nside^2).
/// The angles are in the range [0, pi] and [0, 2 * pi) respectively.
///
/// For details see: https://healpix.sourceforge.io/
///
/// # Arguments
///
///    * `nside` - The resolution parameter.
///    * `pix` - The pixel number.
///
/// # Returns
///
///    * `theta` - The polar angle in radians.
///    * `phi` - The azimuthal angle in radians.
///
pub fn pix2ang_ring(nside: u32, pix: u32) -> (f64, f64) {
    let (z, phi) = pix2ang_ring_z_phi(nside, pix);
    let theta = z.acos();
    (theta, phi)
}

pub fn ang2pix_ring(nside: u32, theta: f64, phi: f64) -> u32 {
    ang2pix_ring_z_phi(nside, theta.cos(), phi)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_healpix() {
        let nside = 16;
        let npix = 12 * nside * nside;
        // Solid angle of a pixel
        let omega = 4.0 * PI / npix as f64;
        let tol = omega.sqrt();
        // Tolerance is the square root of the solid angle of a pixel
        // which is the maximum angle between two points in the same pixel

        let nphi = 256;
        let ntheta = 128;

        (0..nphi).for_each(|i| {
            let phi = 2.0 * PI * i as f64 / nphi as f64;
            (0..ntheta).for_each(|j| {
                let theta = PI * j as f64 / ntheta as f64;
                let pix = ang2pix_ring(nside, theta, phi);
                let (theta2, phi2) = pix2ang_ring(nside, pix);
                // Unit vectors for the point on the sphere
                let v1 = vec![
                    phi.cos() * theta.sin(),
                    phi.sin() * theta.sin(),
                    theta.cos(),
                ];
                // Unit vector for the point on the sphere from the pixel number
                let v2 = vec![
                    phi2.cos() * theta2.sin(),
                    phi2.sin() * theta2.sin(),
                    theta2.cos(),
                ];
                // Angle between the two vectors
                let dot = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum::<f64>();
                let angle = dot.acos();
                // Angle should be less than tolerance
                assert!(angle < tol);
            });
        });
    }
}
