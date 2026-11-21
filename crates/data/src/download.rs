//! Download and cache a Geofabrik .pbf.
use std::{
    error::Error,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub fn cached_path(url: &str, cache_dir: &Path) -> PathBuf {
    let file = url
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("download.osm.pbf");
    cache_dir.join(file)
}

/// Return the cached file, downloading it first if absent.
pub fn ensure_pbf(url: &str, cache_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let path = cached_path(url, cache_dir);
    if path.exists() {
        return Ok(path);
    }
    fs::create_dir_all(cache_dir)?;
    eprintln!("downloading {url}");
    let resp = ureq::get(url).call()?;
    let mut reader = resp.into_body().into_reader();
    let part = path.with_extension("part");
    let mut out = File::create(&part)?;
    let mut buf = vec![0u8; 1 << 20];
    let (mut total, mut next_report) = (0u64, 100u64 << 20);
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n])?;
        total += n as u64;
        if total >= next_report {
            eprintln!("  {} MB", total >> 20);
            next_report += 100 << 20;
        }
    }
    out.flush()?;
    fs::rename(&part, &path)?;
    Ok(path)
}
