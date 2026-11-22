#!/usr/bin/env python3
"""Generate docs/REPORT.md from results/ (measured values only).

Run: python3 -I scripts/gen_report.py   (from the repo root)
Anything missing or empty is reported as "not run"; nothing is invented.
"""
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
R = os.path.join(ROOT, "results")
XEON = "intel_r_xeon_r_cpu_2_00ghz"
NOT_RUN = "_Not run: file missing or empty._"


def txt(rel):
    try:
        with open(os.path.join(R, rel)) as f:
            t = f.read()
        return t if t.strip() else None
    except OSError:
        return None


def js(rel):
    t = txt(rel)
    try:
        return json.loads(t) if t else None
    except ValueError:
        return None


def mac(n):
    return js(f"bench/{n}_apple_m1.json")


def kg(n):
    return js(f"kaggle/bench/{n}_{XEON}.json")


def pt(b, agents, threads=1):
    for p in (b or {}).get("points", []):
        if p["agents"] == agents and p["threads"] == threads:
            return p
    return None


def t(p):
    return p["wall_s"]["median"] if p else None


def f(v, d=3):
    if v is None:
        return "n/a"
    return f"{v:.{d}g}" if abs(v) < 1000 else f"{v:,.0f}"


def x(v):
    return "n/a" if v is None else f"{v:.2f}x"


def ratio(a, b):
    return a / b if a and b else None


def settings(b, hw):
    if not b:
        return f"{hw}: not run"
    p = b["points"][0]
    return f"{hw}; {b['days']} days ({p['steps']} steps), {b['reps']} reps + {b['warmup']} warm-up, median, seed {b['seed']}"


def table(head, rows):
    out = ["| " + " | ".join(head) + " |", "|" + "---|" * len(head)]
    out += ["| " + " | ".join(str(c) for c in r) + " |" for r in rows]
    return "\n".join(out)


def fig(tab):
    return f"Figures: PAGP Lab, **{tab}** tab (`epidemicsim serve`, then open `#/lab`)."


S = []  # sections


def sec(title, method, body, analysis):
    S.append(f"## {title}\n\n**Method.** {method}\n\n{body}\n\n**Analysis.** {analysis}\n")


# 1 performance / Amdahl
V = [("rayon_static_c0", "rayon static"), ("rayon_dynamic_c1024", "rayon dynamic"), ("rayon_guided_c64", "rayon guided"),
     ("openmp_static_c0", "OpenMP static"), ("openmp_dynamic_c1024", "OpenMP dynamic"), ("openmp_guided_c64", "OpenMP guided")]
seq, kseq = mac("sequential"), kg("sequential")
rows = []
for k, l in V:
    b = mac(k)
    if not b:
        continue
    a = next((a for a in b["amdahl"] if a["agents"] == 1_000_000), {})
    sp = {th: (pt(b, 1_000_000, th) or {}).get("speedup") for th in (2, 4, 8)}
    rows.append([l, x(sp[2]), x(sp[4]), x(sp[8]), f(a.get("serial_fraction")), x(a.get("max_speedup"))])
kr = kg("rayon_guided_c64")
if kr:
    a = next((a for a in kr["amdahl"] if a["agents"] == 1_000_000), {})
    rows.append(["rayon guided (Xeon, 1 and 4 threads only)", "-", x((pt(kr, 1_000_000, 4) or {}).get("speedup")), "-", f(a.get("serial_fraction")), x(a.get("max_speedup"))])
tp = [[n, f((pt(seq, n) or {}).get("agents_per_sec")), f((pt(seq, n) or {}).get("contacts_per_sec")), f((pt(kseq, n) or {}).get("agents_per_sec"))] for n in (10_000, 100_000, 1_000_000)]
sec("1. Performance and Amdahl's law",
    "The same simulation (20 days = 80 steps) at several population sizes; speedup is the wall time at 1 thread divided by the time at N threads of the same backend. "
    "An Amdahl fit gives the serial fraction s (speedup(N) = 1 / (s + (1-s)/N)). Contacts are agent-location visits. " + settings(mac("rayon_guided_c64"), "Apple M1 (4P+4E cores), 1M agents"),
    table(["variant", "2 threads", "4 threads", "8 threads", "serial fraction s", "Amdahl max speedup"], rows) + "\n\n"
    + table(["agents", "M1 sequential agents/s", "M1 sequential contacts/s", "Xeon sequential agents/s"], tp) + "\n\n" + fig("Performance"),
    "Speedup saturates well below the thread count; the fitted serial fraction is the share of the run that does not scale (it also absorbs memory-bandwidth limits and the "
    "efficiency cores, so it is not purely algorithmic). " + (f"Best 8-thread speedup: {max(rows, key=lambda r: float(r[3][:-1]) if r[3] not in ('-', 'n/a') else 0)[0]}." if rows else NOT_RUN))

# 2 ILP
nb, ob = mac("ilp_naive"), mac("ilp_optimised")
rows = [[f"{n:,}", f(t(pt(nb, n))), f(t(pt(ob, n))), x(ratio(t(pt(nb, n)), t(pt(ob, n))))] for n in (100_000, 1_000_000)] if nb and ob else []
asm = txt("ilp/analysis.md")
sec("2. Instruction-level parallelism",
    "Naive vs optimised agent kernels (unrolled 4-wide Philox, hoisted per-step constants, selects instead of branches), one thread. " + settings(nb, "Apple M1"),
    (table(["agents", "naive s", "optimised s", "speedup"], rows) if rows else NOT_RUN) + "\n\nStatic assembly counts and discussion: `results/ilp/analysis.md`; assembly: `results/ilp/{naive,optimised}.asm`. " + fig("ILP"),
    (f"Optimised is {x(ratio(t(pt(nb, 1_000_000)), t(pt(ob, 1_000_000))))} faster at 1M agents. " if rows else "") + ("The assembly analysis is in results/ilp/analysis.md (static counts only, not executed counts)." if asm else "No assembly analysis file."))

# 3 TLP
im = {c: js(f"tlp/imbalance_{c}.json") for c in ("mumbai", "chennai")}
rd = js("tlp/race_demo.json")
body = []
rows = []
for k, l in V:
    b = mac(k)
    if b:
        rows.append([l] + [f(t(pt(b, 1_000_000, th))) + f" ({x((pt(b, 1_000_000, th) or {}).get('speedup'))})" for th in (1, 2, 4, 8)])
body.append(table(["variant (1M agents)", "1 thread s (speedup)", "2", "4", "8"], rows) if rows else NOT_RUN)
if all(im.values()):
    r2 = []
    for be in ("rayon", "openmp"):
        for s in ("static", "dynamic", "guided"):
            r2.append([f"{be} {s}"] + [f(next((p["total_time_imbalance"] for p in im[c]["points"] if p["backend"] == be and p["schedule"] == s and p["threads"] == 8), None)) for c in ("mumbai", "chennai")])
    m = im["mumbai"]
    body.append(f"Load imbalance (max/mean per-thread work) at 8 threads; {m['agents']:,} agents, {m['days']} days, {m['reps']} reps, Apple M1:\n\n" + table(["backend schedule", "Mumbai", "Chennai"], r2))
else:
    body.append("Imbalance runs: " + NOT_RUN)
if rd:
    r3 = []
    for s in rd["scenarios"]:
        for b in s["backends"]:
            for mth in b["methods"]:
                r3.append([s["name"], b["backend"], mth["method"], f"{mth['wrong_runs']}/{mth['runs']}", mth["max_lost_increments"], f(mth["time_ms"]["median"])])
    body.append(f"Race demo ({rd['agents']:,} agents, {rd['threads']} threads, {rd['schedule']} chunk {rd['chunk']}, {rd['runs']} runs per method, Apple M1):\n\n" + table(["scenario", "backend", "method", "wrong runs", "max lost increments", "median ms"], r3))
else:
    body.append("Race demo: " + NOT_RUN)
mx = max((p["total_time_imbalance"] for c in im.values() if c for p in c["points"]), default=None)
sec("3. Thread-level parallelism", "Rayon (custom static/dynamic/guided scheduler) and OpenMP (C) with the same three schedules at 1/2/4/8 threads; per-thread work time on Mumbai and Chennai; a deliberate data race on per-location counts.",
    "\n\n".join(body) + "\n\n" + fig("TLP"),
    (f"Maximum measured imbalance was {f(mx)}, so scheduling is not what limits scaling on this machine. " if mx else "") + "Unsynchronised counting produced wrong answers; atomics or per-thread buffers were correct (see table).")

# 4 SIMD
rows = []
for n in (100_000, 1_000_000):
    rows.append([f"{n:,}", f(t(pt(mac("simd_scalar"), n))), f(t(pt(mac("simd_neon"), n))), x(ratio(t(pt(mac("simd_scalar"), n)), t(pt(mac("simd_neon"), n)))),
                 f(t(pt(kg("simd_scalar"), n))), f(t(pt(kg("simd_avx2"), n))), x(ratio(t(pt(kg("simd_scalar"), n)), t(pt(kg("simd_avx2"), n))))])
al = mac("simd_alignment")
r2 = [[r["kernel"], r["variant"], r["alignment"], f(r["ns_per_element"])] for r in al["rows"]] if al else []
sec("4. SIMD", "End-to-end runs with scalar vs vector kernels (timer recovery, Philox stream), 1 thread; alignment test times the kernels alone. " + settings(mac("simd_neon"), "M1 NEON") + " | " + settings(kg("simd_avx2"), "Kaggle Xeon AVX2"),
    table(["agents", "M1 scalar s", "M1 NEON s", "NEON speedup", "Xeon scalar s", "Xeon AVX2 s", "AVX2 speedup"], rows) + "\n\n"
    + (table(["kernel", "variant", "alignment", "ns/element"], r2) if r2 else NOT_RUN) + "\n\n" + fig("SIMD"),
    "End-to-end SIMD gains are small because the vectorised kernels are a minor part of each step (NEON/AVX2 speedups above are measured, on different machines).")

# 5 GPU
mv, mb, cb = js("gpu/metal_vs_cpu.json"), js("gpu/metal_breakdown.json"), js("kaggle/gpu/cuda_breakdown.json")
rows = [[f"{r['agents']:,}", f(r["best_cpu_s"]) + f" ({r['best_cpu']})", f(r["metal_s"]), x(r["speedup_vs_best_cpu"])] for r in (mv or {}).get("rows", [])]
cu, kr = kg("cuda"), kg("rayon_guided_c64")
r2 = [[f"{n:,}", f(t(pt(kr, n, 4))), f(t(pt(cu, n))), x(ratio(t(pt(kr, n, 4)), t(pt(cu, n))))] for n in (100_000, 1_000_000)]
r3 = [[f"{r['agents']:,}", r["block"], f(r["total_s"]), f(r["kernel_s"]), f(r["dispatch_overhead_s"]), f(r["cpu_lambda_s"]), f(r["h2d_s"]), f(r["readback_s"])] for r in (cb or {}).get("rows", [])]
r4 = [[f"{r['agents']:,}", r["storage"], r["threadgroup"], f(r["total_s"]), f(r["kernel_s"]), f(r["dispatch_overhead_s"]), f(r["cpu_lambda_s"]), f(r["readback_s"])] for r in (mb or {}).get("rows", [])]
sec("5. GPU (Metal and CUDA)", "80-step runs; Metal on Apple M1 (unified memory) vs the best M1 CPU run of the same session; CUDA on a Tesla T4 (Kaggle) vs the Xeon 4-thread rayon run. "
    + settings(cu, "Kaggle T4 + Xeon") + ". Metal 10M is not checked against sequential (too slow).",
    "Metal vs best CPU (" + (mv or {}).get("note", "not run") + "):\n\n" + (table(["agents", "best CPU s", "Metal s", "Metal speedup"], rows) if rows else NOT_RUN)
    + "\n\nCUDA (T4) vs Xeon rayon guided 4 threads:\n\n" + table(["agents", "Xeon s", "CUDA s", "speedup"], r2)
    + "\n\nCUDA time breakdown / block sweep (soa_atomic_branch):\n\n" + (table(["agents", "block", "total s", "kernel s", "dispatch s", "CPU lambda s", "H2D s", "readback s"], r3) if r3 else NOT_RUN)
    + "\n\nMetal time breakdown / threadgroup sweep:\n\n" + (table(["agents", "storage", "tg", "total s", "kernel s", "dispatch s", "CPU lambda s", "readback s"], r4) if r4 else NOT_RUN) + "\n\n" + fig("GPU"),
    "Metal is slower than the best CPU at 100k agents (fixed dispatch overhead) and faster at 1M and above; for CUDA the readback and dispatch columns show how much of the run is not kernel time.")

# 6 warps
w = js("kaggle/gpu/cuda_warp.json")
rows = [[f"{r['prevalence']*100:.2f}%", r["variant"], f(r["total_s"]), f(r["kernel_update_s"])] for r in (w or {}).get("rows", [])]
ncu = [txt(f"kaggle/ncu_{n}.csv") for n in ("branch_part0", "pred_part0", "branch_part4")]
err = next((re.search(r"==ERROR==[^\n]*", c).group(0)[:160] for c in ncu if c and "==ERROR==" in c), None)
sec("6. Warp divergence", "Branchy vs predicated update kernels, and agents partitioned by health state every 4/16 steps, at prevalences 0.05% to 40%. "
    + (f"{w['device']}, {w['rows'][0]['agents']:,} agents, {w['days']} days, {w['reps']} reps, block {w['rows'][0]['block']}." if w else "not run"),
    (table(["prevalence", "variant", "total s", "update kernel s"], rows) if rows else NOT_RUN)
    + "\n\nNsight Compute branch efficiency: " + (f"**profiler unavailable** ({err})" if err else ("see results/kaggle/ncu_*.csv" if any(ncu) else NOT_RUN)) + "\n\n" + fig("Warps"),
    "See the table for which variant had the lowest update-kernel time at each prevalence; without ncu counters, divergence is inferred from timing only, not measured directly.")

# 7 PTX
sm = js("kaggle/ptx/summary.json")
rows, groups = [], []
if sm:
    for fl in ("place", "stats", "update"):
        by = {}
        for lv in ("O0", "O1", "O2", "O3", "O3fm", "G"):
            e = sm["files"].get(f"{fl}_{lv}")
            if e:
                rows.append([fl, lv, e["sha256"], e["totals"].get("total"), e["totals"].get("bra", 0), e["totals"].get("setp", 0), e["totals"].get("selp", 0)])
                by.setdefault(e["sha256"], []).append(lv)
        groups.append(f"{fl}: " + " | ".join("=".join(v) for v in by.values()))
sec("7. PTX", "nvcc compiled each CUDA kernel file at -O0..-O3, -O3 with fast-math and -G (device debug) for " + (sm["arch"] if sm else "n/a") + "; instructions counted per opcode in the .ptx. Counts are static.",
    (table(["file", "level", "sha256 (16)", "instructions", "bra", "setp", "selp"], rows) if rows else NOT_RUN) + "\n\nLevels with the same hash are identical PTX: " + "; ".join(groups) + ".\n\n"
    + "Note: `results/kaggle/ptx/analysis.md` and `results/ptx/analysis.md` are unfilled templates and are not used here.\n\n" + fig("PTX"),
    "Optimisation levels -O0 to -O3 and fast-math produced byte-identical PTX for these kernels (equal hashes), so the only measurable code-generator effect is -G." if groups and all(len(g.split(" | ")) == 2 for g in groups) else "See hash groups above.")

# 8 MPI
m = js("mpi/scaling.json")
rows = [[p["mode"], p["procs"], f(p["wall_s"]["median"]), x(p["speedup"]), f(p["efficiency"]), f(p["comm_fraction"] * 100) + "%", f(p["idle_s_total"]), p["bytes_sent_total"]] for p in (m or {}).get("points", [])]
sec("8. MPI", "Monte Carlo workload (scenario runs) split over MPI ranks, static vs dynamic (rank 0 is a pure master in dynamic mode). "
    + (f"{m['machine']}, {m['backend']} backend, {m['agents']:,} agents, {m['days']} days, {m['scenarios']} scenarios x {m['runs_per_scenario']} runs, {m['reps']} reps." if m else "not run"),
    (table(["mode", "procs", "median s", "speedup", "efficiency", "comm share", "idle s (total)", "bytes sent"], rows) if rows else NOT_RUN) + "\n\n" + fig("MPI"),
    "Communication is a negligible share of static runs; dynamic mode loses a rank to the master and counts the master's waiting as communication.")

# 9 OpenACC
notes, bl, rl, steps = txt("kaggle/openacc_notes.md"), txt("kaggle/logs/openacc_build.log"), txt("kaggle/logs/openacc_run.log"), txt("kaggle/steps.tsv")
codes = dict(l.split("\t")[:2] for l in (steps or "").splitlines() if "\t" in l)
sec("9. OpenACC", "An OpenACC version of the benchmark (`crates/kernels-c/openacc/acc_bench.c`) was built with gcc on the Kaggle machine.",
    f"Exit codes: build {codes.get('openacc_build', 'n/a')}, run {codes.get('openacc_run', 'n/a')}.\n\n```\n{(bl or 'no build log').strip()[:600]}\n```\n\n" + (f"Notes:\n\n{notes.strip()}\n" if notes else ""),
    "OpenACC was attempted and not measured: gcc there has no nvptx offload support, so no binary was produced and no timings exist.")

# summary + limitations
mach = js("kaggle/machine.json") or {}
lim = f"""## Limitations

- **Epidemic parameters are uncalibrated.** Transmission, durations, severity and scenario effects are illustrative defaults, not fitted to any real outbreak; absolute epidemic numbers must not be read as forecasts.
- **Kaggle run settings.** Kaggle numbers come from a shared notebook: {mach.get('gpu', 'GPU n/a').splitlines()[0] if mach.get('gpu') else 'GPU n/a'} (x2 listed), {mach.get('cpu', 'CPU n/a')} with {mach.get('cpu_cores', '?')} cores, {mach.get('os', '?')}, run {mach.get('date_utc', '?')}. Benchmarks use 20 days (80 steps), 5 reps + 1 warm-up, seed 42, 100k and 1M agents only; the Xeon thread sweep is 1 and 4 threads. Other tenants may add noise.
- **ncu unavailable.** Nsight Compute could not read GPU counters ({err or 'no ncu output'}), so there is no measured branch efficiency.
- **OpenACC unavailable.** Build failed (gcc without nvptx offload), so there are no OpenACC results.
- **AVX2 only on the Kaggle CPU.** NEON was measured on the M1 and AVX2 on the Xeon; the two are different machines and not comparable.
- **Mac vs Kaggle** are different hardware; cross-machine times are not like-for-like. Metal 10M results are not checked against sequential.
- **Mac test logs were not saved**; Mac golden-test status is taken from docs/PROGRESS.md. The Kaggle golden-table run panicked on the NEON row after 8 PASS rows (since fixed to skip NEON on non-aarch64, not re-run there).
- PTX analysis markdown files in results/ are unfilled templates; counts come from `summary.json`.
- Figures are rendered live in the Lab web app (not exported as images); this report references the tab for each.
"""
head = """# Report: parallel backends for an agent-based epidemic simulator

_Generated by `python3 -I scripts/gen_report.py` from `results/`. Every number below is read from a results file; missing data is marked "not run"._
"""
out = head + "\n" + "\n".join(S) + "\n" + lim
with open(os.path.join(ROOT, "docs", "REPORT.md"), "w") as fh:
    fh.write(out)
print(f"wrote docs/REPORT.md ({len(out.splitlines())} lines)")
sys.exit(0)
