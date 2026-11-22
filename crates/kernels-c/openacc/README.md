# OpenACC (bonus)

`acc_bench.c`: standalone C program, **not** part of the Cargo build and not wired into the
Rust simulation. `acc_update` (`#pragma acc parallel loop`, Philox draw + integer thresholds,
fixed durations) and `acc_stats` (`reduction(+:c0..c6)`), checked against a serial loop in the
same binary.

- NVHPC: `nvc -acc -gpu=cc75 -O3 -Minfo=accel acc_bench.c -o acc_bench`
- GCC offload (Ubuntu `gcc-offload-nvptx`): `gcc -O3 -fopenacc -foffload=nvptx-none acc_bench.c -o acc_bench`
- Host only (pragmas ignored): `cc -O2 acc_bench.c -o acc_bench` (verified on macOS: logic runs,
  matches serial). Nothing here has run on a GPU.

`scripts/kaggle_run.sh` tries the build when `nvc` or the GCC offload compiler is present and
writes `results/kaggle/openacc_notes.md` either way.
