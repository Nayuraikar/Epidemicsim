Read CLAUDE.md. Do NOT implement any simulation logic yet. Only scaffold:

1. Create a Cargo workspace (resolver 2) with these crates under crates/: core, data, scenarios, backends, bench, server, cli. Give each an empty lib.rs (cli gets a main.rs that prints "epidemicsim" and uses clap with subcommands run, compare, bench, pipeline, serve, all stubbed).
2. Create crates/kernels-c/ as a crate with a build.rs using the `cc` crate and placeholder folders openmp/ and cuda/ each containing a README.md describing the planned kernels. build.rs must compile cleanly on macOS with no kernels yet.
3. Add rust-toolchain.toml (stable), a rustfmt.toml, and a clippy-clean CI-style script scripts/check.sh that runs fmt, clippy and tests.
4. Create web/ using Vite + React + TypeScript (npm create vite), light theme only, with a placeholder landing page. Don't build features yet.
5. Add empty cities/*.toml for bengaluru, mumbai, delhi, chennai with TODO fields, and scenarios/*.toml for baseline, masks, vax_random, vax_targeted, isolation, closures, lockdown, combo with TODO fields.
6. Write a first docs/PROGRESS.md with a checklist of the 8 phases and an empty "Findings" section. Write a stub docs/SCHEMA.md.
7. Verify with `cargo build --workspace` and `cargo test --workspace`, then summarise what exists.

Working rules (save tokens):
- Be concise. No long summaries; end each task with <=10 lines: what changed, test status, next step.
- Don't re-read files you just wrote. Don't paste large files or logs back; show only errors.
- Run tests with quiet output (`cargo test -q`, `cargo build -q`).
- Don't refactor or add features beyond the current prompt.
- Update docs/PROGRESS.md with <=5 lines per phase.
- Never run git commit, git push, git add, git reset or git checkout. The user commits. When a task is finished, print a suggested commit message and stop. Ask before adding dependencies.