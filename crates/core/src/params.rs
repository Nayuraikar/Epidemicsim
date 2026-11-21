//! Simulation parameters (TOML-loadable; every field has a default).
use serde::{Deserialize, Serialize};

pub const N_AGE: usize = 5;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct DiseaseParams {
    /// Per-step transmission coefficient by location kind: [home, work, market, hospital].
    pub beta: [f64; 4],
    pub rel_infectiousness_asymptomatic: f64,
    /// P(asymptomatic | infected), by age group.
    pub p_asymptomatic: [f64; N_AGE],
    pub latent_days: f64,
    pub asymptomatic_days: f64,
    pub symptomatic_days: f64,
    pub hospital_days: f64,
    /// P(hospitalised | symptomatic, unvaccinated), by age group.
    pub p_hospitalised: [f64; N_AGE],
    /// P(death | hospitalised), by age group.
    pub p_death_hospitalised: [f64; N_AGE],
    /// Fraction of agents infected (state E) at step 0.
    pub initial_infected_frac: f64,
}

impl Default for DiseaseParams {
    fn default() -> Self {
        Self {
            beta: [0.03, 0.02, 0.02, 0.0],
            rel_infectiousness_asymptomatic: 0.5,
            p_asymptomatic: [0.5, 0.4, 0.3, 0.25, 0.2],
            latent_days: 3.0,
            asymptomatic_days: 5.0,
            symptomatic_days: 5.0,
            hospital_days: 10.0,
            p_hospitalised: [0.01, 0.03, 0.08, 0.18, 0.30],
            p_death_hospitalised: [0.02, 0.05, 0.12, 0.25, 0.40],
            initial_infected_frac: 0.0005,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct VaccineParams {
    /// Peak efficacy against infection.
    pub ve_infection: f64,
    /// Peak efficacy against hospitalisation given symptomatic infection.
    pub ve_severe: f64,
    /// Days to ramp linearly from 0 to peak efficacy.
    pub ramp_days: f64,
    /// Exponential waning half-life (days) after the ramp.
    pub waning_half_life_days: f64,
}

impl Default for VaccineParams {
    fn default() -> Self {
        Self {
            ve_infection: 0.7,
            ve_severe: 0.9,
            ramp_days: 14.0,
            waning_half_life_days: 180.0,
        }
    }
}

impl VaccineParams {
    /// Fraction of peak efficacy remaining `days` after vaccination.
    pub fn waning_factor(&self, days: u16) -> f64 {
        let d = f64::from(days);
        let ramp = if self.ramp_days > 0.0 {
            (d / self.ramp_days).min(1.0)
        } else {
            1.0
        };
        ramp * 0.5f64.powf((d - self.ramp_days).max(0.0) / self.waning_half_life_days)
    }

    pub fn eff_infection(&self, status: u8, days: u16) -> f64 {
        if status == 0 {
            0.0
        } else {
            self.ve_infection * self.waning_factor(days)
        }
    }

    pub fn eff_severe(&self, status: u8, days: u16) -> f64 {
        if status == 0 {
            0.0
        } else {
            self.ve_severe * self.waning_factor(days)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SynthParams {
    /// Cumulative distribution of household size 1..=6.
    pub household_size_cdf: [f64; 6],
    /// Cumulative distribution over age groups.
    pub age_cdf: [f64; N_AGE],
    /// P(attends school / has a workplace) by age group.
    pub employment: [f64; N_AGE],
    pub school_size: f64,
    pub workplace_size: f64,
    pub agents_per_market: f64,
    pub agents_per_hospital: f64,
    /// Fraction vaccinated at step 0 (days since vaccination uniform in 0..365).
    pub initial_vax_frac: f64,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            household_size_cdf: [0.10, 0.30, 0.55, 0.80, 0.92, 1.0],
            age_cdf: [0.27, 0.57, 0.79, 0.93, 1.0],
            employment: [0.8, 0.75, 0.75, 0.35, 0.05],
            school_size: 200.0,
            workplace_size: 25.0,
            agents_per_market: 500.0,
            agents_per_hospital: 25_000.0,
            initial_vax_frac: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub seed: u64,
    pub agents: usize,
    pub days: u32,
    pub disease: DiseaseParams,
    pub vaccine: VaccineParams,
    pub synth: SynthParams,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 42,
            agents: 100_000,
            days: 120,
            disease: Default::default(),
            vaccine: Default::default(),
            synth: Default::default(),
        }
    }
}

impl Config {
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::from_toml_str(&std::fs::read_to_string(path)?)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_toml_uses_defaults() {
        let c = Config::from_toml_str("seed = 7\n[vaccine]\nve_infection = 0.5\n").unwrap();
        assert_eq!(c.seed, 7);
        assert_eq!(c.vaccine.ve_infection, 0.5);
        assert_eq!(c.agents, 100_000);
    }

    #[test]
    fn vaccine_ramps_then_wanes() {
        let v = VaccineParams::default();
        assert_eq!(v.eff_infection(0, 30), 0.0);
        assert_eq!(v.waning_factor(0), 0.0);
        assert!((v.waning_factor(14) - 1.0).abs() < 1e-12);
        assert!((v.waning_factor(14 + 180) - 0.5).abs() < 1e-12);
        assert!(v.waning_factor(300) < v.waning_factor(200));
    }
}
