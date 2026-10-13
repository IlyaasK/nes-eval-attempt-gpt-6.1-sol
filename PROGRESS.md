# Continuing work

The user's goal is indefinite improvement of video and audio for arbitrary
input sequences. The goal remains active. No internet access is permitted.
No subagents have been used. Build from the repository root with:

```
cargo build --release --lib --target wasm32-unknown-unknown
```

The current artifact is `target/wasm32-unknown-unknown/release/nes_emu.wasm`.
The wasm has no host imports. `tools/wasm_check.py` tests all ABI calls except
reset via a WAT harness preloaded alongside the wasm. `tests/abi.rs` tests
cold reset, stable ROM buffer allocation, deterministic output, and draining.
Native and wasm audio/framebuffer hashes match on a 660-frame input replay.

## Architecture

- `src/cpu.rs`: asynchronous instruction execution, including unofficial
  opcodes and bus dummy accesses. Futures queue one bus operation at a time.
  `Nes::clock` executes it, then polls the future through its pure effects.
  `Bus` owns PPU/APU/cartridge/RAM and controller state.
- `src/ppu.rs`: dot-clocked background shifts/fetches and sprite evaluation/
  fetches. Sprite zero hit, overflow diagonal scan, scroll increment and
  reload, palette aliases, grayscale/emphasis, open-bus decay and warmup.
- `src/apu.rs`: pulse, triangle, noise, DMC, frame counter and IRQs.
- `src/audio.rs`: nonlinear mixer output resampled by fractional-phase
  Blackman-windowed sinc delta synthesis, then 90/440 Hz high passes and
  14 kHz low pass. This is approximate, not bit-identical to the oracle.
- `src/cart.rs`: iNES/NES 2.0 mappers 0/1/2/3/4, mapper RAM requirements,
  MMC1 serial filtering, 512 KiB outer banking, MMC3 A12 IRQ filtering.

## Reference discoveries

- First `emu_run_frame` advances no cycles. Verified with an original ROM
  that reads input at its first instructions, changes input at frame 1,
  and renders the result. The first call's audio is empty.
- Reference framebuffer is updated in place and a normal CPU instruction
  finishes after the PPU frame wraps. The first few pixels of the next frame
  can therefore be visible in the returned image. Using a double buffer
  caused small top-row differences; direct rendering mostly fixes them.
- CPU NMI sampling after the first PPU tick of each CPU cycle passes vblank
  suppression/control tests. Dispatching NMI from the current latched state
  at instruction end (including branches) also matches the reference's
  interrupt test and avoids File Fixers noise SFX divergence. The older
  branch-specific polling implementation caused large audio errors.
- CPU vector selection after pushing status, using the current NMI latch,
  produces the reference's interrupt-hijacking result table.
- PPU status read at scanline 241 dot 1 suppresses the pending vblank set.
- Odd skip uses a rendering snapshot from pre-render dot 338. Inverting the
  frame parity still passed the even/odd diagnostics, but worsened some
  idle video comparisons; current parity is the original `frame & 1 != 0`.
- MMC3 sprite A12 rises on dot 260, background A12 on 324. The visible
  scanline's dot 0 CHR address must also be presented to avoid false IRQ
  clocks; presenting it on pre-render dot 0 made the bundled test fail.
- $4017 delayed reset is 3 cycles for even CPU cycles, 4 for odd, counting
  from after the write. All bundled APU length/IRQ/jitter tests pass.
- DMC rate index 13 is 84, not 85. DMC timer's power-on value is 419 after
  the eight reset cycles. Noise LFSR/timer are initialized after reset too:
  shift 0x4000, timer 1. The reference clocks noise on the half-rate APU
  clock and reloads the listed period directly (a reference compatibility
  difference from the full CPU-period hardware table).
- Audio output delay is 17 samples; source clock phase starts after eight
  reset cycles. A 20,000-frame no-render probe constrained the reference
  sample-clock CPU frequency to 1789772.94--1789773.06 Hz. It is 1,789,773
  Hz, now used by the resampler. This removes a three-sample long-run drift.
- OAM GET parity currently uses odd numbered CPU cycles. Inverting it
  worsened File Fixers/Stallar audio, so that experiment was reverted.
  DMC alignment currently fetches on even numbered CPU cycles and still
  needs independent phase investigation.

## Validation

`cargo test`: 10 unit tests and 1 ABI integration test pass. Added tests for
mapper banking/RAM/mirroring, malformed size overflow, SHX address/data
masking on page carry, and partial OAM DMA across a frame boundary.

`tools/regress.py` ran every bundled test at 600 frames (1800 for instruction
timing). All RAM-reported diagnostics passed. CPU reset appropriately stops
at status 0x81 requesting the hardware Reset button; ABI reset is cold boot.
The sprite overflow timing final frame matched the oracle byte-for-byte.
Other sprite diagnostics' final screens were spot checked; full automated
oracle comparison of all screen-only tests remains useful.

Original 660-frame replay `tools/gameplay.txt` exercises Start, A, held
right/up, left/down, and A+B. All four bundled games progress beyond their
titles. Video was over 99.999% exact in those traces, with almost all remaining
differences a handful of overwritten pixels at row 0. Exact-frame counts at
an earlier checkpoint: robotfindskitten 659/660, RHDE 657/660, File Fixers
650/660, Stallar 644/660. Small later timing fixes may change these counts.

Audio at that checkpoint: robotfindskitten RMSE below 1 PCM unit, RHDE about
6; after fixing branch NMI polling, File Fixers approximately 12.5 PCM units
and correlation 0.99993. Stallar remains about 595 PCM units / 0.807
correlation, because its noise stream diverges after about 3.5 seconds.
Sample count matches the oracle: 483551 samples for the 660-frame replays.

## Next investigations

1. Stallar audio: noise is the differing channel. Muting only noise in a
   native debug run and subtracting it from oracle audio showed matching
   non-noise channels. After 3.5 sec, noise correlations collapse. Replaying
   timestamped native noise-register writes in an original synthetic ROM
   (`tools/replay_noise.py`) matches the oracle closely through this interval.
   This indicates a CPU/PPU/DMA timing difference around the game's writes,
   rather than a noise algorithm error. Branch-NMI polling fixes File Fixers,
   but not Stallar. Audio writes a cycle either side of a timer reload can
   change the later LFSR sequence. Inspect write timings/interrupt latency.
2. Unusual OAM DMA I/O reads: the original `dma_boundary_probe.s` DMA's page
   $40 while controller strobe is high. Native OAM[22] contains A; oracle
   reports zero even for the early-DMA variant. Normal CPU reads with strobe
   high, and normal OAMDATA reads, both correctly report 1 in a separate
   oracle probe. Thus the reference's DMA path appears to handle I/O reads
   differently (possibly a side-effect-free peek). A palette transition also
   differs by one frame for the boundary probe, plausibly a polling-phase
   vblank suppression. Do not assume the controller result alone proves
   DMA scheduling; the early-DMA experiment disproved that inference.
3. Exact FIR/resampling. `tools/fit_dac.py` fits generalized impulse response
   from repeated DAC transitions. Current sinc beta 0.9206, radius 16, normal
   delta offset 1.334 samples gave very close pulse/triangle/DAC outputs.
   It is not exact, especially initial transient and high-frequency noise.
4. Wider input replays, mapper 2/3 ROM probes, MMC1 board variants, DMC/OAM
   overlap/IRQ/controller read conflicts, and PPUDATA/OAM reads while rendering.

## Local diagnostic artifacts

Everything under `.out/` is generated and ignored by git. There are many
reference `.idx` frames, WAVs, native raw audio, and JSON/log traces there.
Important directories: `input-ref-{rfk,rhde,ff,stallar}`, matching native
`input-*`, `live-*`, `branch-*`, and `regression/results.json`.
`events-stallar.txt`/`events-game.txt` contain `WRITE <cycle> <addr> <value>`
and `MIX <cycle> <p1> <p2> <tri> <noise> <dmc>` from the native `trace` feature.
`NES_MUTE=8` mutes only noise for debugging. `NES_NOISE_SEED` and
`NES_NOISE_TIMER` alter noise initial state in debug builds only.
Original assembly probes are in `tools/`; compile with ca65 and ld65 using
`nrom.cfg` (16 KiB) or `nrom32.cfg` (32 KiB with DMC sample segment).

rustfmt is absent and must not be downloaded. Rust source was made readable
using the compiler's AST pretty printer, solely as an offline formatting tool:
`RUSTC_BOOTSTRAP=1 rustc -Zunpretty=normal --edition 2021 file.rs > .out/pretty.rs`.
The required cargo build needs no custom RUSTFLAGS or configuration.
