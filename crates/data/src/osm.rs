//! Extract location centroids from an OSM .pbf, clipped to a bbox.
use crate::geo::Bbox;
use osmpbf::{Element, ElementReader};
use std::{error::Error, path::Path};

#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub lon: f64,
    pub lat: f64,
    /// Homes: residential weight (building levels); others: 1.
    pub weight: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Pois {
    pub homes: Vec<Site>,
    pub offices: Vec<Site>,
    pub schools: Vec<Site>,
    pub colleges: Vec<Site>,
    pub markets: Vec<Site>,
    pub hospitals: Vec<Site>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Home,
    Office,
    School,
    College,
    Market,
    Hospital,
}

#[derive(Default)]
struct Tags<'a> {
    building: Option<&'a str>,
    amenity: Option<&'a str>,
    shop: Option<&'a str>,
    healthcare: Option<&'a str>,
    office: bool,
    levels: Option<f64>,
}

fn collect<'a>(it: impl Iterator<Item = (&'a str, &'a str)>) -> Tags<'a> {
    let mut t = Tags::default();
    for (k, v) in it {
        match k {
            "building" => t.building = Some(v),
            "amenity" => t.amenity = Some(v),
            "shop" => t.shop = Some(v),
            "healthcare" => t.healthcare = Some(v),
            "office" => t.office = true,
            "building:levels" => t.levels = v.parse().ok(),
            _ => {}
        }
    }
    t
}

fn classify(t: &Tags) -> Option<(Kind, f64)> {
    if t.amenity == Some("hospital") || t.healthcare == Some("hospital") {
        return Some((Kind::Hospital, 1.0));
    }
    match t.amenity {
        Some("college" | "university") => return Some((Kind::College, 1.0)),
        Some("school") => return Some((Kind::School, 1.0)),
        Some("marketplace") => return Some((Kind::Market, 1.0)),
        _ => {}
    }
    if matches!(
        t.shop,
        Some(
            "supermarket" | "mall" | "department_store" | "convenience" | "greengrocer" | "general"
        )
    ) {
        return Some((Kind::Market, 1.0));
    }
    if t.office {
        return Some((Kind::Office, 1.0));
    }
    if t.shop.is_some() || t.amenity.is_some() {
        return None;
    }
    match t.building? {
        "commercial" | "office" | "industrial" => Some((Kind::Office, 1.0)),
        "retail" | "supermarket" => Some((Kind::Market, 1.0)),
        "yes" | "residential" | "apartments" | "house" | "detached" | "semidetached_house"
        | "terrace" | "dormitory" | "hut" | "bungalow" => {
            Some((Kind::Home, t.levels.unwrap_or(1.0).clamp(1.0, 40.0)))
        }
        _ => None,
    }
}

impl Pois {
    fn push(&mut self, kind: Kind, site: Site) {
        match kind {
            Kind::Home => self.homes.push(site),
            Kind::Office => self.offices.push(site),
            Kind::School => self.schools.push(site),
            Kind::College => self.colleges.push(site),
            Kind::Market => self.markets.push(site),
            Kind::Hospital => self.hospitals.push(site),
        }
    }
}

/// Node coordinates inside the bbox, in id order (PBF nodes are normally sorted).
#[derive(Default)]
struct NodeStore {
    ids: Vec<i64>,
    lonlat: Vec<(f64, f64)>,
    sorted: bool,
}

impl NodeStore {
    fn add(&mut self, id: i64, lon: f64, lat: f64) {
        if self.ids.last().is_some_and(|&l| l >= id) {
            self.sorted = false;
        }
        self.ids.push(id);
        self.lonlat.push((lon, lat));
    }

    fn finish(&mut self) {
        if !self.sorted {
            let mut idx: Vec<usize> = (0..self.ids.len()).collect();
            idx.sort_unstable_by_key(|&i| self.ids[i]);
            self.ids = idx.iter().map(|&i| self.ids[i]).collect();
            self.lonlat = idx.iter().map(|&i| self.lonlat[i]).collect();
            self.sorted = true;
        }
    }

    fn get(&self, id: i64) -> Option<(f64, f64)> {
        self.ids.binary_search(&id).ok().map(|i| self.lonlat[i])
    }
}

pub fn extract(pbf: &Path, bbox: &Bbox) -> Result<Pois, Box<dyn Error>> {
    let mut pois = Pois::default();
    let mut nodes = NodeStore {
        sorted: true,
        ..Default::default()
    };
    let mut finished = false;
    let add_poi = |pois: &mut Pois, tags: Tags, lon: f64, lat: f64| {
        if let Some((k, w)) = classify(&tags) {
            pois.push(
                k,
                Site {
                    lon,
                    lat,
                    weight: w,
                },
            );
        }
    };
    ElementReader::from_path(pbf)?.for_each(|el| match el {
        Element::Node(n) => {
            let (lon, lat) = (n.lon(), n.lat());
            if bbox.contains(lon, lat) {
                nodes.add(n.id(), lon, lat);
                add_poi(&mut pois, collect(n.tags()), lon, lat);
            }
        }
        Element::DenseNode(n) => {
            let (lon, lat) = (n.lon(), n.lat());
            if bbox.contains(lon, lat) {
                nodes.add(n.id(), lon, lat);
                add_poi(&mut pois, collect(n.tags()), lon, lat);
            }
        }
        Element::Way(w) => {
            if !finished {
                nodes.finish();
                finished = true;
            }
            let tags = collect(w.tags());
            if classify(&tags).is_none() {
                return;
            }
            let (mut sx, mut sy, mut n) = (0.0, 0.0, 0u32);
            for r in w.refs() {
                if let Some((lon, lat)) = nodes.get(r) {
                    sx += lon;
                    sy += lat;
                    n += 1;
                }
            }
            if n > 0 {
                let (lon, lat) = (sx / f64::from(n), sy / f64::from(n));
                if bbox.contains(lon, lat) {
                    add_poi(&mut pois, tags, lon, lat);
                }
            }
        }
        Element::Relation(_) => {}
    })?;
    Ok(pois)
}
