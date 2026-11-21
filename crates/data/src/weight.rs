//! Population weighting: how likely each residential building is to host a household.
use crate::osm::Site;
use std::{error::Error, fs::File, io::BufReader, path::Path};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;

pub trait PopWeight {
    fn name(&self) -> &'static str;
    /// One non-negative weight per home building.
    fn weights(&self, homes: &[Site]) -> Vec<f64>;
}

/// Building density: each residential building weighs its levels.
pub struct BuildingDensity;

impl PopWeight for BuildingDensity {
    fn name(&self) -> &'static str {
        "building_density"
    }
    fn weights(&self, homes: &[Site]) -> Vec<f64> {
        homes.iter().map(|h| h.weight).collect()
    }
}

/// WorldPop raster (north-up, EPSG:4326). A pixel's population is split across its buildings.
pub struct WorldPop {
    pub width: usize,
    pub height: usize,
    pub lon0: f64,
    pub lat0: f64,
    pub dx: f64,
    pub dy: f64,
    pub pop: Vec<f32>,
}

impl WorldPop {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut d = Decoder::new(BufReader::new(File::open(path)?))?;
        let (w, h) = d.dimensions()?;
        let scale = d.get_tag_f64_vec(Tag::ModelPixelScaleTag)?;
        let tie = d.get_tag_f64_vec(Tag::ModelTiepointTag)?;
        if scale.len() < 2 || tie.len() < 6 {
            return Err("GeoTIFF lacks pixel-scale/tiepoint tags".into());
        }
        let pop = match d.read_image()? {
            DecodingResult::F32(v) => v,
            DecodingResult::F64(v) => v.into_iter().map(|x| x as f32).collect(),
            _ => return Err("WorldPop raster must be float".into()),
        };
        Ok(Self {
            width: w as usize,
            height: h as usize,
            lon0: tie[3] - tie[0] * scale[0],
            lat0: tie[4] + tie[1] * scale[1],
            dx: scale[0],
            dy: scale[1],
            pop,
        })
    }

    fn pixel(&self, lon: f64, lat: f64) -> Option<usize> {
        let x = ((lon - self.lon0) / self.dx).floor();
        let y = ((self.lat0 - lat) / self.dy).floor();
        (x >= 0.0 && y >= 0.0 && (x as usize) < self.width && (y as usize) < self.height)
            .then(|| y as usize * self.width + x as usize)
    }
}

impl PopWeight for WorldPop {
    fn name(&self) -> &'static str {
        "worldpop"
    }
    fn weights(&self, homes: &[Site]) -> Vec<f64> {
        let px: Vec<Option<usize>> = homes.iter().map(|h| self.pixel(h.lon, h.lat)).collect();
        let mut per_pixel = std::collections::HashMap::<usize, f64>::new();
        for (p, h) in px.iter().zip(homes) {
            if let Some(p) = p {
                *per_pixel.entry(*p).or_default() += h.weight;
            }
        }
        let w: Vec<f64> = px
            .iter()
            .zip(homes)
            .map(|(p, h)| match p {
                // nodata is negative in WorldPop rasters
                Some(p) => f64::from(self.pop[*p].max(0.0)) * h.weight / per_pixel[p],
                None => 0.0,
            })
            .collect();
        if w.iter().sum::<f64>() > 0.0 {
            w
        } else {
            BuildingDensity.weights(homes)
        }
    }
}

/// WorldPop if a raster is configured/present, else building density.
pub fn choose(path: Option<&Path>) -> Result<Box<dyn PopWeight>, Box<dyn Error>> {
    match path {
        Some(p) if p.exists() => Ok(Box::new(WorldPop::from_file(p)?)),
        _ => Ok(Box::new(BuildingDensity)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worldpop_splits_pixel_population_over_buildings() {
        let r = WorldPop {
            width: 2,
            height: 1,
            lon0: 0.0,
            lat0: 1.0,
            dx: 1.0,
            dy: 1.0,
            pop: vec![100.0, 10.0],
        };
        let s = |lon| Site {
            lon,
            lat: 0.5,
            weight: 1.0,
        };
        let w = r.weights(&[s(0.2), s(0.4), s(1.5), s(5.0)]);
        assert_eq!(w, vec![50.0, 50.0, 10.0, 0.0]);
    }
}
