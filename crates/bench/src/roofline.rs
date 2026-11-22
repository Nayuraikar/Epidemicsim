//! CPU roofline: memory bandwidth (STREAM-style triad), peak FP32 FMA throughput (NEON on
//! aarch64), peak scalar integer throughput, and the two SIMD kernels' measured position.
use crate::{hardware_tag, machine_info, thermal_note, timing, tlp::write_json, Machine, Timing};
use epidemicsim_backends::simd::{self, AlignedBuf};
use serde::Serialize;
use std::{error::Error, hint::black_box, path::Path, time::Instant};

#[derive(Serialize)]
pub struct Roof {
    pub name: String,
    pub threads: usize,
    /// GB/s for bandwidth roofs, Gop/s for compute roofs; the summary uses the best repetition
    /// (peak), mean/median/CI describe the spread.
    pub peak: f64,
    pub unit: &'static str,
    pub value: Timing,
    pub detail: String,
}

#[derive(Serialize)]
pub struct KernelPoint {
    pub kernel: &'static str,
    pub variant: &'static str,
    pub elements: usize,
    pub working_set_mb: f64,
    pub ns_per_element: Timing,
    /// Bytes read + written per element (from the kernel source).
    pub bytes_per_element: f64,
    /// Integer ops per element (static tally of the scalar source, same for the vector build).
    pub int_ops_per_element: f64,
    pub flops_per_element: f64,
    /// ops per byte.
    pub arithmetic_intensity: f64,
    pub achieved_gb_s: f64,
    pub achieved_gops: f64,
}

#[derive(Serialize)]
pub struct RooflineReport {
    pub machine: Machine,
    pub reps: usize,
    pub thermal_before: Option<String>,
    pub thermal_after: Option<String>,
    pub notes: Vec<&'static str>,
    pub roofs: Vec<Roof>,
    pub kernels: Vec<KernelPoint>,
    pub gpu: Option<String>,
}

fn run_threads(threads: usize, f: impl Fn(usize) + Sync) {
    std::thread::scope(|s| {
        for t in 0..threads {
            let f = &f;
            s.spawn(move || f(t));
        }
    });
}

/// a[i] = b[i] + s * c[i] over f64 arrays split across threads; 24 bytes per element.
fn triad(elems: usize, threads: usize, reps: usize) -> Roof {
    let mut a = vec![0.0f64; elems];
    let b = vec![1.0f64; elems];
    let c = vec![2.0f64; elems];
    let chunk = elems.div_ceil(threads);
    let mut gbs = Vec::new();
    for rep in 0..reps + 1 {
        let t0 = Instant::now();
        std::thread::scope(|s| {
            for ((a, b), c) in a
                .chunks_mut(chunk)
                .zip(b.chunks(chunk))
                .zip(c.chunks(chunk))
            {
                s.spawn(move || {
                    for i in 0..a.len() {
                        a[i] = b[i] + 3.0 * c[i];
                    }
                });
            }
        });
        let secs = t0.elapsed().as_secs_f64();
        if rep > 0 {
            gbs.push(24.0 * elems as f64 / secs / 1e9);
        }
    }
    black_box(&a);
    let value = timing(&gbs);
    Roof {
        name: "memory bandwidth (STREAM triad, f64)".into(),
        threads,
        peak: value.max,
        unit: "GB/s",
        value,
        detail: format!(
            "{} MB per array x 3, 24 B/element (no write-allocate counted)",
            elems * 8 / 1_000_000
        ),
    }
}

#[cfg(target_arch = "aarch64")]
fn fma_kernel(iters: u64) -> f32 {
    use std::arch::aarch64::*;
    // SAFETY: NEON is mandatory on aarch64.
    unsafe {
        let m = vdupq_n_f32(black_box(1.000_000_1));
        let c = vdupq_n_f32(black_box(1e-7));
        let mut acc = [vdupq_n_f32(1.0); 12];
        for _ in 0..iters {
            for a in acc.iter_mut() {
                *a = vfmaq_f32(c, *a, m);
            }
        }
        acc.iter().map(|a| vaddvq_f32(*a)).sum()
    }
}
#[cfg(not(target_arch = "aarch64"))]
fn fma_kernel(iters: u64) -> f32 {
    let (m, c) = (black_box(1.000_000_1f32), black_box(1e-7f32));
    let mut acc = [1.0f32; 32];
    for _ in 0..iters {
        for a in acc.iter_mut() {
            *a = a.mul_add(m, c);
        }
    }
    acc.iter().sum()
}

fn int_kernel(iters: u64) -> u64 {
    let (m, c) = (
        black_box(0x9E37_79B9_7F4A_7C15u64),
        black_box(0x1234_5678u64),
    );
    let mut acc = [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    for _ in 0..iters {
        for a in acc.iter_mut() {
            *a = a.wrapping_mul(m).wrapping_add(c);
        }
    }
    acc.iter().fold(0, |x, y| x ^ y)
}

fn compute_roof(
    name: &str,
    unit_ops_per_iter: f64,
    threads: usize,
    reps: usize,
    f: fn(u64) -> u64,
    iters: u64,
    detail: &str,
) -> Roof {
    let mut g = Vec::new();
    for rep in 0..reps + 1 {
        let t0 = Instant::now();
        run_threads(threads, |_| {
            black_box(f(iters));
        });
        let secs = t0.elapsed().as_secs_f64();
        if rep > 0 {
            g.push(unit_ops_per_iter * iters as f64 * threads as f64 / secs / 1e9);
        }
    }
    let value = timing(&g);
    Roof {
        name: name.into(),
        threads,
        peak: value.max,
        unit: "Gop/s",
        value,
        detail: detail.into(),
    }
}

fn kernel_points(reps: usize, out: &mut Vec<KernelPoint>) {
    let k_neon = simd::SimdKind::Neon.check().is_ok();
    let mut kinds = vec![simd::SimdKind::Scalar];
    if k_neon {
        kinds.push(simd::SimdKind::Neon);
    }
    for n in [1usize << 20, 1 << 25] {
        let passes = if n <= 1 << 20 { 20 } else { 2 };
        let mut stt = AlignedBuf::<u8>::new(n);
        let mut tm = AlignedBuf::<u16>::new(n);
        let mut ids = AlignedBuf::<u32>::new(n);
        let mut o = AlignedBuf::<u32>::new(n);
        for i in 0..n {
            stt.as_mut_slice()[i] = 1 + (i % 4) as u8;
            tm.as_mut_slice()[i] = u16::MAX;
            ids.as_mut_slice()[i] = i as u32;
        }
        for &k in &kinds {
            for kernel in ["timer_recovery", "philox_stream"] {
                let mut ns = Vec::new();
                for rep in 0..reps + 1 {
                    let mut trans = Vec::new();
                    let t0 = Instant::now();
                    for _ in 0..passes {
                        if kernel == "timer_recovery" {
                            simd::timer_dispatch(k, stt.as_slice(), tm.as_mut_slice(), &mut trans);
                        } else {
                            simd::stream_dispatch(k, ids.as_slice(), 3, [42, 0], o.as_mut_slice());
                        }
                    }
                    if rep > 0 {
                        ns.push(t0.elapsed().as_secs_f64() * 1e9 / (passes * n) as f64);
                    }
                }
                let t = timing(&ns);
                // Source tallies. timer: load state (1 B), load+store timer (4 B); ops: state-1,
                // compare, saturating decrement, compare to 0, push test. philox: load id, store
                // word (8 B); 10 rounds x (2 mul, 2 shift, 4 xor) + 9 x 2 key adds = 98.
                let (bytes, ops) = if kernel == "timer_recovery" {
                    (5.0, 5.0)
                } else {
                    (8.0, 98.0)
                };
                out.push(KernelPoint {
                    kernel: if kernel == "timer_recovery" {
                        "timer_recovery"
                    } else {
                        "philox_stream"
                    },
                    variant: k.name(),
                    elements: n,
                    working_set_mb: n as f64 * bytes / 1e6,
                    bytes_per_element: bytes,
                    int_ops_per_element: ops,
                    flops_per_element: 0.0,
                    arithmetic_intensity: ops / bytes,
                    achieved_gb_s: bytes / t.median,
                    achieved_gops: ops / t.median,
                    ns_per_element: t,
                });
            }
        }
    }
}

pub fn run(out_dir: &Path, reps: usize) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let thermal_before = thermal_note();
    let mut roofs = Vec::new();
    for t in [1, 4, 8] {
        roofs.push(triad(16 << 20, t, reps));
        eprintln!(
            "triad {t} threads: peak {:.1} GB/s",
            roofs.last().unwrap().peak
        );
    }
    for t in [1, 8] {
        let fl = if cfg!(target_arch = "aarch64") {
            12.0 * 4.0 * 2.0
        } else {
            32.0 * 2.0
        };
        let mut r = compute_roof(
            "peak FP32 FMA",
            fl,
            t,
            reps,
            |i| fma_kernel(i).to_bits() as u64,
            50_000_000,
            if cfg!(target_arch = "aarch64") {
                "NEON vfmaq_f32, 12 independent accumulators"
            } else {
                "scalar f32 mul_add, 32 chains (compiler-vectorised)"
            },
        );
        r.unit = "GFLOP/s";
        eprintln!("fma {t} threads: peak {:.1} GFLOP/s", r.peak);
        roofs.push(r);
    }
    for t in [1, 8] {
        let r = compute_roof(
            "peak scalar integer mul+add (u64)",
            2.0 * 12.0,
            t,
            reps,
            int_kernel,
            50_000_000,
            "12 independent wrapping_mul+wrapping_add chains",
        );
        eprintln!("int {t} threads: peak {:.1} Gop/s", r.peak);
        roofs.push(r);
    }
    let mut kernels = Vec::new();
    kernel_points(reps, &mut kernels);
    let machine = machine_info();
    let report = RooflineReport {
        reps,
        thermal_before,
        thermal_after: thermal_note(),
        notes: vec![
            "Roofs are measured (best repetition). Kernel bytes/ops per element are static tallies from the kernel source, not hardware counters.",
            "Both SIMD kernels are integer kernels (no floating-point work), so they are placed against the integer roof, not the FP32 roof.",
            "The 1M-element working sets fit in cache, so their achieved GB/s can exceed the DRAM triad bandwidth; the 32M-element rows are DRAM-resident.",
            "Kernels run on one thread.",
            "The FMA and integer roofs come from fixed-length dependent-chain loops (12 independent chains); they are measured achievable rates and may sit below the true hardware peak.",
        ],
        roofs,
        kernels,
        gpu: None,
        machine: machine.clone(),
    };
    let _ = hardware_tag(&machine); // single-machine file name requested: mac.json
    let path = out_dir.join("mac.json");
    write_json(&path, &report)?;
    Ok(path)
}
