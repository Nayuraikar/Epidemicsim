# Calibration (rough)

**This is a rough calibration to an assumed target. It is not a forecast and not a fit to real data.**

## Target (assumed, no citation)
- Early doubling time of daily new infections of about **5 days** (illustrative, not a citation).
- Asymptomatic share of infections of about **0.40** (illustrative, not a citation).
- Both are placeholders so the model is in a plausible-looking growth regime; nothing was checked against surveillance data.

## Method
- Parameters fitted: a scale on the home/work/market betas (hospital beta stays 0) and a scale on every age's P(asymptomatic) (capped at 0.95).
- Growth rate = log-linear regression on daily new infections from day 6 until day 30 or 5% cumulative attack, whichever is first (without the cut-off, depletion made doubling time non-monotonic in beta; the first attempt picked an artefact and was discarded).
- Loss = ((doubling - 5) / 0.5)^2 + ((asym share - 0.40) / 0.05)^2.
- Grid: 100,000 synthetic agents, 5 runs per point (seeds 42-46), sequential backend. Coarse: beta x {1, 1.5, 2, 2.5, 3, 3.5, 4, 5} by asymptomatic x {0.6, 0.8, 1.0, 1.1, 1.2, 1.4}; fine: beta in steps of 0.1 around the best coarse point. Full grid: `results/calibration/grid.json`; rerun with `epidemicsim calibrate`.
- Synthetic rather than a Bengaluru sample: the 1M city graphs are not subsampled here, and `subsample_world` was only tuned on Mumbai.

## Fitted values (`config/calibrated.toml`, use `--params calibrated`)
- beta x1.6: [0.048, 0.032, 0.032, 0] (default [0.03, 0.02, 0.02, 0]).
- P(asymptomatic) x1.1: [0.55, 0.44, 0.33, 0.275, 0.22]; population-weighted share 0.407.
- Fit error: doubling time 4.98 +/- 0.21 days (sd over 5 runs) vs target 5.0; share 0.407 vs 0.40. Default parameters give 12.1 days.
- The target does not identify both parameters: at beta x1.5, doubling time is 4.6-6.0 days across asymptomatic scales 0.6-1.4, so the asymptomatic fraction is mostly set by its own assumed target.

## Transfer to the real city graphs (`results/calibration/transfer.json`)
Doubling time, same fit, 1M agents, 5 seeds: Mumbai 4.59-4.72 d, Bengaluru 5.40-5.56 d. Within a 4-6 day band although fitted on synthetic cities.

## compare, 20 runs, 8 scenarios, 120 days (`results/<city>_calibrated/`; old results kept)
| city | params | R initial | baseline attack | isolation attack |
|---|---|---|---|---|
| Mumbai | default (50 runs) | 1.33 | 41.7% | 2.2% |
| Mumbai | calibrated | 2.06 | 72.9% | 48.8% |
| Bengaluru | calibrated | 1.96 | 71.2% | 37.9% |

Attack rates near 70% follow from a 5-day doubling time in this SEIR structure; they are not an estimate for either city.

## Isolation under calibrated parameters
- With default parameters isolation (60% of symptomatic agents stay home from day 7) cut the attack rate from 41.7% to 2.2% in Mumbai. That was mostly a threshold effect: R was about 1.33, so a modest cut pushed it below 1.
- Calibrated, the effect is still large but no longer near-elimination: Mumbai 72.9% to 48.8%, Bengaluru 71.2% to 37.9% (peak A+I down 58% and 69%). On the 100k synthetic sample compliance 0.48 / 0.60 / 0.72 gives attack rates of 52.6 / 44.3 / 32.2%.
- Why it is still large (reading of the code, not a separate experiment): in the state machine E is not infectious and I is infectious from onset (`backend.rs`), so symptom onset and infectiousness coincide. Isolation (`schedule.rs`) acts on state I immediately, with no presymptomatic transmission and no detection delay, so it is effectively perfect case-finding for the compliant share. Symptomatic agents carry about 74% of infectiousness at the fitted asymptomatic share (symptomatic 0.593 x 5 d = 2.97 vs asymptomatic 0.407 x 0.5 x 5 d = 1.02 weighted days), and 60% compliance removes their non-home contacts, so up to about 45% of non-home infectiousness. Household transmission continues.
- Verdict: the dominant cause is model structure (instant, perfect symptom detection), then the symptomatic share, then the compliance setting. Real isolation effects would be smaller; treat the isolation scenario as an upper bound.

## Not covered
Only growth rate was fitted: not deaths, hospital load, peak timing or serial interval. One target, assumed. Not validated on any outbreak.
