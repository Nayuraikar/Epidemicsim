# Running the CUDA / warp / PTX / AVX2 benchmarks on Kaggle

Nothing CUDA-related has run yet (no NVIDIA GPU locally). This is the exact procedure for the
first run. Expect to fix small things the first time (see "If something fails").

## 1. Notebook settings
- New Notebook, language **Python** (you only use `!` shell cells).
- Right sidebar -> Session options: **Accelerator: GPU P100** (or **GPU T4 x2**; one GPU is used),
  **Internet: On**, Persistence: none.
- Check you got a GPU: cell `!nvidia-smi`.

## 2. Get the repo into the notebook (pick one)
a. **Dataset (recommended).** On your Mac, in the repo root:
   `zip -r epidemicsim.zip . -x 'target/*' 'web/node_modules/*' 'web/dist/*' 'data_cache/*' '*.pbf' '.git/*' 'results/*'`
   Kaggle -> Datasets -> New Dataset -> upload `epidemicsim.zip` (private). In the notebook:
   Add Input -> your dataset. Kaggle may extract the zip; the script handles both a folder and a zip.
b. **Git.** If the repo is on GitHub: set `REPO_URL=https://github.com/<you>/<repo>` in step 3.
Never upload the large `.pbf` files; the run uses a synthetic city generated in-process.

## 3. Run (one cell each, in order)
```
# smoke test first (~ a few minutes after the Rust build): fewer reps and agents
!QUICK=1 bash $(find /kaggle/input -name kaggle_run.sh | head -1)
```
Zip-only dataset (nothing extracted):
```
!mkdir -p /kaggle/working/src && unzip -q /kaggle/input/*/epidemicsim.zip -d /kaggle/working/src && bash /kaggle/working/src/scripts/kaggle_run.sh
```
Git variant: `!REPO_URL=... bash <(curl -fsSL https://raw.githubusercontent.com/<you>/<repo>/main/scripts/kaggle_run.sh)`.
Then the full run (drop `QUICK=1`; 1M agents, 5 reps): the same command. 
The script (see its header): installs rustup, builds `--features cuda` (no MPI), runs the golden
tests (`EPI_REQUIRE_CUDA=1`: a missing GPU or any non-matching variant fails loudly), a 200k-agent
synthetic run on `sequential` vs `cuda`, then `bench --suite cuda | cuda-variants | warp`, CPU
baselines on the same machine, AVX2 (`simd_avx2_<cpu>.json`), `ncu` if installed, and `scripts/ptx.sh`. Each step logs to `results/kaggle/logs/<step>.log`; the exit code of every
step is in `results/kaggle/steps.tsv`; one failure never stops the rest.

## 4. Collect results
- Output tab (or `/kaggle/working/epidemicsim_kaggle_results.zip`): download it.
- Unzip into the repo's `results/` (it contains `kaggle/`). Key files:
  `kaggle/machine.json`, `kaggle/gpu/cuda_breakdown.json` (block sweep, kernel / H2D / readback
  split, same shape as `results/gpu/metal_breakdown.json`), `kaggle/gpu/cuda_variants.json`
  (atomics vs segmented, branch/pred/smem/pairs, AoS vs SoA), `kaggle/gpu/cuda_warp.json`
  (divergence variants x prevalence), `kaggle/bench/simd_avx2_*.json`, `kaggle/ptx/summary.json`,
  `kaggle/ncu_*.csv`.
- Every row has `matches_sequential`; a `false` is a correctness bug in that variant, not a result.
- Fill `results/ptx/analysis.md` (template) with the timings.

## If something fails
- **Build error in `cudarc`:** the crate is pinned to the `cuda-12040` bindings in
  `crates/backends/Cargo.toml`. Match your toolkit (`nvcc --version`; available features include
  `cuda-12060`, `cuda-12080`, `cuda-13000`, ...) and re-run.
- **NVRTC compile error:** the log (`logs/test_cuda_unit.log`) prints the compiler message; kernels
  have never been compiled, so syntax/semantic slips are plausible. Send me the log.
- **`ncu` ERR_NVGPUCTRPERM / not installed:** expected on Kaggle; recorded, nothing else affected.
- **`apt` / disk / timeout:** raise `STEP_TIMEOUT`; the steps are independent.
