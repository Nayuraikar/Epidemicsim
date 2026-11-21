//! Scenario files (`scenarios/<name>.toml`), see docs/SCHEMA.md.
use serde::{Deserialize, Serialize};
use std::{error::Error, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    /// (A + I) / population.
    Infectious,
    /// Hospital occupancy / capacity.
    Hospital,
    /// 1 - S / population.
    Cumulative,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Trigger {
    pub metric: Metric,
    pub above: f64,
}

/// Activation window: starts on `start_day` or when `trigger` first fires (default: day 0),
/// and ends after `duration_days` (default: never).
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct When {
    #[serde(default)]
    pub start_day: Option<u32>,
    #[serde(default)]
    pub trigger: Option<Trigger>,
    #[serde(default)]
    pub duration_days: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    Random,
    /// Only the listed age groups are eligible.
    Age,
    /// Blocks of homes (spatially contiguous) ranked by current infections, highest first.
    Location,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Intervention {
    /// Transmission multiplied by `1 - compliance * efficacy`.
    Masks {
        #[serde(flatten)]
        when: When,
        compliance: f64,
        efficacy: f64,
    },
    /// Vaccinate up to `coverage` of the population at `rate_per_day` of the population per day.
    Vaccination {
        #[serde(flatten)]
        when: When,
        strategy: Strategy,
        coverage: f64,
        #[serde(default = "default_rate")]
        rate_per_day: f64,
        /// Eligible age groups for `age` (default 60+).
        #[serde(default = "default_ages")]
        ages: Vec<u8>,
        /// Homes per block for `location`.
        #[serde(default = "default_block")]
        block_homes: u32,
    },
    /// Symptomatic agents stay home with probability `compliance`.
    Isolation {
        #[serde(flatten)]
        when: When,
        compliance: f64,
    },
    CollegeClosure {
        #[serde(flatten)]
        when: When,
    },
    MarketClosure {
        #[serde(flatten)]
        when: When,
    },
    /// Work/school/college and markets closed; everyone stays home.
    Lockdown {
        #[serde(flatten)]
        when: When,
    },
}

fn default_rate() -> f64 {
    0.01
}
fn default_ages() -> Vec<u8> {
    vec![3, 4]
}
fn default_block() -> u32 {
    1000
}

impl Intervention {
    pub fn when(&self) -> &When {
        match self {
            Intervention::Masks { when, .. }
            | Intervention::Vaccination { when, .. }
            | Intervention::Isolation { when, .. }
            | Intervention::CollegeClosure { when }
            | Intervention::MarketClosure { when }
            | Intervention::Lockdown { when } => when,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub interventions: Vec<Intervention>,
}

fn unit(x: f64, what: &str) -> Result<(), String> {
    if (0.0..=1.0).contains(&x) {
        Ok(())
    } else {
        Err(format!("{what} must be in [0, 1], got {x}"))
    }
}

impl Scenario {
    pub fn from_toml_str(s: &str) -> Result<Self, Box<dyn Error>> {
        let sc: Scenario = toml::from_str(s)?;
        sc.validate()?;
        Ok(sc)
    }

    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let s = std::fs::read_to_string(path)?;
        Self::from_toml_str(&s).map_err(|e| format!("{}: {e}", path.display()).into())
    }

    pub fn validate(&self) -> Result<(), String> {
        for iv in &self.interventions {
            let w = iv.when();
            if w.start_day.is_some() && w.trigger.is_some() {
                return Err(format!("{}: set start_day or trigger, not both", self.name));
            }
            if let Some(t) = &w.trigger {
                if t.above < 0.0 {
                    return Err(format!("{}: trigger.above must be >= 0", self.name));
                }
            }
            match iv {
                Intervention::Masks {
                    compliance,
                    efficacy,
                    ..
                } => {
                    unit(*compliance, "masks.compliance")?;
                    unit(*efficacy, "masks.efficacy")?;
                }
                Intervention::Isolation { compliance, .. } => unit(*compliance, "compliance")?,
                Intervention::Vaccination {
                    coverage,
                    rate_per_day,
                    ages,
                    block_homes,
                    ..
                } => {
                    unit(*coverage, "vaccination.coverage")?;
                    unit(*rate_per_day, "vaccination.rate_per_day")?;
                    if ages.iter().any(|&a| a >= 5) {
                        return Err("vaccination.ages must be age groups 0..=4".into());
                    }
                    if *block_homes == 0 {
                        return Err("vaccination.block_homes must be > 0".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// Load every `*.toml` in `dir`, sorted by name with `baseline` first.
pub fn load_all(dir: &Path) -> Result<Vec<Scenario>, Box<dyn Error>> {
    let mut v = Vec::new();
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = e?.path();
        if p.extension().is_some_and(|x| x == "toml") {
            v.push(Scenario::from_file(&p)?);
        }
    }
    v.sort_by(|a, b| (a.name != "baseline", &a.name).cmp(&(b.name != "baseline", &b.name)));
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flattened_timing_and_defaults() {
        let s = Scenario::from_toml_str(
            r#"
name = "x"
[[interventions]]
kind = "masks"
trigger = { metric = "infectious", above = 0.01 }
duration_days = 30
compliance = 0.5
efficacy = 0.4
[[interventions]]
kind = "vaccination"
strategy = "age"
start_day = 5
coverage = 0.3
[[interventions]]
kind = "lockdown"
"#,
        )
        .unwrap();
        assert_eq!(s.interventions.len(), 3);
        let w = s.interventions[0].when();
        assert_eq!(w.duration_days, Some(30));
        assert_eq!(w.trigger.unwrap().metric, Metric::Infectious);
        match &s.interventions[1] {
            Intervention::Vaccination {
                when,
                rate_per_day,
                ages,
                ..
            } => {
                assert_eq!(when.start_day, Some(5));
                assert_eq!(*rate_per_day, 0.01);
                assert_eq!(ages, &vec![3, 4]);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn rejects_bad_values() {
        let bad = "name='x'\n[[interventions]]\nkind='isolation'\ncompliance=1.5\n";
        assert!(Scenario::from_toml_str(bad).is_err());
        let both = "name='x'\n[[interventions]]\nkind='lockdown'\nstart_day=1\ntrigger={metric='hospital',above=0.5}\n";
        assert!(Scenario::from_toml_str(both).is_err());
    }
}
