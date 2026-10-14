# NES Eval attempt: GPT-6.1 Sol in codex, high thinking, 24 hours

One run of [NES Eval](https://github.com/IlyaasK/nes-eval): a model gets an
empty Rust project, the NES Eval task, and a black-box reference emulator,
and has 24 hours to write an NES emulator that compiles to WebAssembly.
This repository is the model's working directory, saved every 15 minutes,
plus the transcript and the grades. It follows the layout of GBA Eval's
attempt repositories (e.g.
[gba-eval-attempt-claude-opus-5](https://github.com/mechanize-work/gba-eval-attempt-claude-opus-5)).

| | |
|---|---|
| Model | `gpt-6.1-sol` |
| Thinking | `high` |
| Agent harness | codex CLI 0.162.1, `codex exec --json` |
| Environment | NES Eval `quickstart/` task container (commit `12af20f`) |
| Run length | 24 hours, 2026-10-13 19:44:10 to 2026-10-14 19:46:34 UTC |
| Saves | every 15 minutes, 96 commits |
| Grades | every 4 hours plus a final one: 6 checkpoints, all built and graded |

## Score

| Checkpoint | Overall | Procedural | Replay | Audio |
|---|---|---|---|---|
| 4 h | 0.9776 | 0.8901 | 1.0000 | 0.9980 |
| 8 h | 0.9860 | 0.9300 | 1.0000 | 0.9999 |
| 12 h | 0.9860 | 0.9300 | 1.0000 | 1.0000 |
| 16 h | 0.9860 | 0.9300 | 1.0000 | 1.0000 |
| 20 h | 0.9860 | 0.9300 | 1.0000 | 1.0000 |
| **final** | **0.9860** | 0.9300 | 1.0000 | 1.0000 |

**28 of the 29 testcases score at or above 0.999.** Every one of the six
homebrew games replays frame-exactly: all 34,000 replayed frames match the
reference pixel for pixel. Audio is exact.

The one failure is `nmi-and-brk` (blargg's `cpu_interrupts_v2/2`, NMI
hijacking a BRK), which scores 0.0: its output first differs at frame 110
and diverges for 100 of the 210 graded frames. That single testcase is the
entire remaining gap — 0.07 of the 0.20 procedural weight, 1.4% of the
overall score.

For scale, on the same grader: the reference scores 1.0000, NES Eval's
hand-written baseline `nes-core` 0.5811, Space Bunny Free 0.0594 in 2 hours,
and the Qwen3.8 27B run never compiled.

## What it built

7,199 lines of Rust, 96 commits, organised by subsystem:

| File | Lines | What |
|---|---|---|
| `src/cpu.rs` | 1,533 | every official and unofficial 6502 instruction, with per-bus-cycle scheduling |
| `src/ppu.rs` | 1,486 | dot-clocked background fetches, scrolling, secondary OAM evaluation, sprite zero/overflow, palette and open-bus behaviour |
| `src/apu.rs` | 790 | pulse, triangle, noise, DMC, frame counter and IRQs |
| `src/cart.rs` | 585 | mappers 0–4, iNES and NES 2.0, CHR RAM, MMC1 serial/RMW filtering, MMC3 A12 IRQ |
| `src/audio_kernel.rs`, `src/audio_phase_table.rs`, `src/audio.rs` | 2,713 | nonlinear mixer plus calibrated polyphase FIR resampling and filters |
| `src/lib.rs` | 86 | the wasm ABI from `spec/ABI.md` |

It also wrote its own test harness (`tests/abi.rs` for cold reset, ROM
allocation, determinism and audio draining; `tools/wasm_check.py` to drive
the exports under wasmtime), a native runner (`examples/run.rs`), and 44
Python tools under `tools/` for audio and PPU analysis. It used the oracle
105 times to run ROMs and 17 times to inspect sessions.

Its own `PROGRESS.md`, which it kept updated throughout, ends with itself
working on a residual OAM row-corruption bug and the note *"Task not
complete; keep improving. No agent delegation / no internet used."*

## How the run went

Five codex invocations across the 24 hours; no API failures and no rate-limit
errors. The first ran 1h15m and exited on its own, and the supervisor resumed
it with "continue". The watchdog then stopped three in a row after 15 minutes
of silence each (the third had run 11.6 hours straight), and the fifth ran to
the deadline. No work was lost across restarts: codex resumed its session with
`exec resume --last`, and the supervisor had already committed everything.

Token use: 227,945,482 input tokens (222,900,992 of them cache reads),
1,875,649 output tokens, of which 1,310,683 were reasoning tokens.

## Files

- `src/`, `Cargo.toml`, `tests/`, `tools/`, `examples/`: the emulator, as of
  its last commit, including the model's own `README.md` and `PROGRESS.md`.
- `TASK.md`, `spec/`, `dev-roms/`: the task environment as handed to the
  model (first commit).
- `transcripts/transcript.md`: the conversation, all 1,250 tool calls with
  their commands and outputs, and the 7 context compactions (long outputs cut
  to 4,000 characters). Rendered with NES Eval's
  `scripts/render-codex-transcript.py`.
- `transcripts/session.jsonl.xz`: the complete codex rollout (40 MB,
  uncompressed). Codex stores reasoning only as encrypted content, so no
  readable chain of thought is in there; the transcript notes each elision.
- `grading/`: `run.json` (per-checkpoint commits, build and grade results),
  the supervisor log, each checkpoint's `summary.json`, and the final
  checkpoint's end-screen images (`<test>.ref.png` next to
  `<test>.cand.png`). `nmi-and-brk.ref.png` vs `nmi-and-brk.cand.png` is the
  one visible difference.

Two account identifiers that codex writes into its rollout metadata
(`creator_user_id`, `creator_account_id`) are replaced with
`<redacted-account-id>`. No credentials appear anywhere in this repository.

## Reproduce the grade

```sh
git clone https://github.com/IlyaasK/nes-eval-attempt-gpt-6.1-sol && cd nes-eval-attempt-gpt-6.1-sol
cargo build --release --lib --target wasm32-unknown-unknown
# then, from a NES Eval checkout:
target/release/grader <path>/target/wasm32-unknown-unknown/release/nes_emu.wasm corpus/ out/gpt-6.1-sol
```

The grader is deterministic, so this reproduces 0.9860.
