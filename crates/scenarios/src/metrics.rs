//! Per-run metrics and distribution summaries across runs.
use epidemicsim_core::agents::state;
use epidemicsim_core::world::STEPS_PER_DAY;
use epidemicsim_core::StepStats;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct RunMetrics {
    /// Peak of A + I.
    pub peak_infections: f64,
    pub peak_day: f64,
    /// 1 - S_end / N.
    pub attack_rate: f64,
    pub deaths: f64,
    pub peak_hospital_load: f64,
    /// Peak hospital load / capacity.
    pub peak_hospital_ratio: f64,
    /// Days (steps / 4) with hospital load above capacity.
    pub days_over_capacity: f64,
    /// Mean R_t over the first 7 days (steps with infectious agents).
    pub r_initial: f64,
    /// Maximum daily-mean R_t.
    pub r_max: f64,
    pub vaccine_doses: f64,
}

impl RunMetrics {
    pub const NAMES: [&'static str; 10] = [
        "peak_infections",
        "peak_day",
        "attack_rate",
        "deaths",
        "peak_hospital_load",
        "peak_hospital_ratio",
        "days_over_capacity",
        "r_initial",
        "r_max",
        "vaccine_doses",
    ];

    pub fn values(&self) -> [f64; 10] {
        [
            self.peak_infections,
            self.peak_day,
            self.attack_rate,
            self.deaths,
            self.peak_hospital_load,
            self.peak_hospital_ratio,
            self.days_over_capacity,
            self.r_initial,
            self.r_max,
            self.vaccine_doses,
        ]
    }

    pub fn from_stats(stats: &[StepStats], agents: usize, capacity: u32, doses: u64) -> Self {
        let spd = STEPS_PER_DAY as usize;
        let (mut peak, mut peak_step, mut peak_h) = (0u32, 0u32, 0u32);
        let mut over = 0usize;
        for s in stats {
            let inf = s.counts[state::A as usize] + s.counts[state::I as usize];
            if inf > peak {
                (peak, peak_step) = (inf, s.step);
            }
            peak_h = peak_h.max(s.hospital_load);
            over += usize::from(s.hospital_load > capacity);
        }
        let last = stats.last().map(|s| s.counts).unwrap_or([0; 7]);
        let first_week: Vec<f64> = stats
            .iter()
            .take(7 * spd)
            .map(|s| s.r_t)
            .filter(|&r| r > 0.0)
            .collect();
        let r_max = stats
            .chunks(spd)
            .map(|c| c.iter().map(|s| s.r_t).sum::<f64>() / c.len() as f64)
            .fold(0.0, f64::max);
        Self {
            peak_infections: f64::from(peak),
            peak_day: f64::from(peak_step / STEPS_PER_DAY),
            attack_rate: 1.0 - f64::from(last[state::S as usize]) / agents.max(1) as f64,
            deaths: f64::from(last[state::D as usize]),
            peak_hospital_load: f64::from(peak_h),
            peak_hospital_ratio: f64::from(peak_h) / f64::from(capacity.max(1)),
            days_over_capacity: over as f64 / spd as f64,
            r_initial: if first_week.is_empty() {
                0.0
            } else {
                first_week.iter().sum::<f64>() / first_week.len() as f64
            },
            r_max,
            vaccine_doses: doses as f64,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct Dist {
    pub mean: f64,
    pub p05: f64,
    pub p95: f64,
}

/// Linear-interpolated quantile of an ascending-sorted slice.
pub fn quantile(sorted: &[f64], q: f64) -> f64 {
    match sorted.len() {
        0 => 0.0,
        1 => sorted[0],
        n => {
            let pos = q * (n - 1) as f64;
            let lo = pos.floor() as usize;
            let hi = pos.ceil() as usize;
            sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
        }
    }
}

impl Dist {
    pub fn of(values: &[f64]) -> Self {
        let mut v = values.to_vec();
        v.sort_by(f64::total_cmp);
        Dist {
            mean: v.iter().sum::<f64>() / v.len().max(1) as f64,
            p05: quantile(&v, 0.05),
            p95: quantile(&v, 0.95),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantiles_interpolate() {
        let v: Vec<f64> = (0..=100).map(f64::from).collect();
        assert_eq!(quantile(&v, 0.05), 5.0);
        assert_eq!(quantile(&v, 0.95), 95.0);
        let d = Dist::of(&[1.0, 2.0, 3.0]);
        assert_eq!(d.mean, 2.0);
        assert!((d.p05 - 1.1).abs() < 1e-12);
    }

    #[test]
    fn metrics_from_synthetic_series() {
        let mk = |step: u32, i: u32, h: u32, s: u32, d: u32| StepStats {
            step,
            day: step / 4,
            counts: [s, 0, 0, i, h, 0, d],
            new_infections: 0,
            hospital_load: h,
            r_t: 2.0,
        };
        let stats = vec![
            mk(0, 1, 0, 90, 0),
            mk(1, 5, 3, 80, 0),
            mk(2, 2, 6, 70, 4),
            mk(3, 0, 1, 60, 4),
        ];
        let m = RunMetrics::from_stats(&stats, 100, 2, 7);
        assert_eq!(m.peak_infections, 5.0);
        assert_eq!(m.peak_day, 0.0);
        assert_eq!(m.attack_rate, 0.4);
        assert_eq!(m.deaths, 4.0);
        assert_eq!(m.peak_hospital_load, 6.0);
        assert_eq!(m.peak_hospital_ratio, 3.0);
        assert_eq!(m.days_over_capacity, 0.5);
        assert_eq!(m.r_initial, 2.0);
        assert_eq!(m.vaccine_doses, 7.0);
    }
}
