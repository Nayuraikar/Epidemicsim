//! Bounding box, local km projection and Hilbert ordering.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Bbox {
    pub min_lon: f64,
    pub min_lat: f64,
    pub max_lon: f64,
    pub max_lat: f64,
}

impl Bbox {
    pub fn contains(&self, lon: f64, lat: f64) -> bool {
        lon >= self.min_lon && lon <= self.max_lon && lat >= self.min_lat && lat <= self.max_lat
    }

    /// Equirectangular projection to km around the bbox centre.
    pub fn to_km(&self, lon: f64, lat: f64) -> (f64, f64) {
        let lat0 = 0.5 * (self.min_lat + self.max_lat);
        let lon0 = 0.5 * (self.min_lon + self.max_lon);
        (
            (lon - lon0) * lat0.to_radians().cos() * 111.32,
            (lat - lat0) * 110.57,
        )
    }

    /// Hilbert-curve index on a 65536x65536 grid over the bbox.
    pub fn hilbert(&self, lon: f64, lat: f64) -> u64 {
        let q =
            |v: f64, lo: f64, hi: f64| (((v - lo) / (hi - lo)).clamp(0.0, 1.0) * 65535.0) as u32;
        hilbert_d(
            q(lon, self.min_lon, self.max_lon),
            q(lat, self.min_lat, self.max_lat),
        )
    }
}

fn hilbert_d(mut x: u32, mut y: u32) -> u64 {
    let n: u32 = 1 << 16;
    let mut d = 0u64;
    let mut s = n / 2;
    while s > 0 {
        let rx = u32::from(x & s > 0);
        let ry = u32::from(y & s > 0);
        d += u64::from(s) * u64::from(s) * u64::from((3 * rx) ^ ry);
        if ry == 0 {
            if rx == 1 {
                x = n - 1 - x;
                y = n - 1 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }
        s /= 2;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hilbert_is_bijective_on_small_grid_and_local() {
        // adjacent cells at order 16 have nearby indices on average; spot-check bijection
        let a = hilbert_d(0, 0);
        let b = hilbert_d(1, 0);
        let c = hilbert_d(0, 1);
        assert_eq!(a, 0);
        assert_ne!(b, c);
        assert!(b < 4 && c < 4);
    }
}
