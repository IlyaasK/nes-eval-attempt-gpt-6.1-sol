# Continuing work

The user's goal is indefinite improvement of video and audio for arbitrary
input sequences. The goal remains active. No internet access is permitted.
No subagents have been used. Build from the repository root with:

```
cargo build --release --lib --target wasm32-unknown-unknown
```

The artifact is `target/wasm32-unknown-unknown/release/nes_emu.wasm`.
The emulator has no host imports. `tools/wasm_check.py` exercises the exported
ABI under wasmtime; `tests/abi.rs` checks cold reset, stable ROM allocation,
determinism and draining. Do not mark the task complete.

## Architecture

- `src/cpu.rs`: all official/unofficial instructions and individual bus
  accesses. Instructions own a Cpu clone and Rc<RefCell<Bus>> in futures;
  the scheduler polls a future, executes one queued bus cycle, then polls
  through pure CPU effects to the next request or completed instruction.
- `src/ppu.rs`: dot-clocked background shifts/fetches, scrolling, secondary
  OAM evaluation/fetches, sprite zero/overflow, palette/open bus/warmup.
- `src/apu.rs`: pulse, triangle, noise, DMC, frame counter and IRQs.
- `src/audio.rs`: nonlinear mixer, fractional-phase Blackman-window sinc
  delta synthesis and HP90/HP440/LP14k filters. Approximate, not exact yet.
- `src/cart.rs`: mappers 0/1/2/3/4, NES1/2, CHR RAM, RAM requirements,
  MMC1 serial/RMW filtering/512 KiB outer bank, MMC3 inversion/A12 IRQ.

## Latest verified timing fixes (supersede the old checkpoint assumptions)

Earlier experiments changed these independently and appeared to regress
some games. Correcting them together resolved the large game audio errors.
Do not revert them because an old isolated experiment looked worse.

1. OAM DMA GET is on even numbered CPU cycles (PUT odd). A write completed
   on an even cycle stalls 513 cycles; an odd write stalls 514. An original
   NMI/DMA/DAC marker ROM now matches all 57 reference timestamps exactly.
2. Both pulse channels and noise clock on odd numbered CPU cycles. Noise
   clocking even had a one-cycle error near mode writes; pulse channel 2
   eventually diverged at a period reload near six seconds in Stallar.
   Isolating each channel and changing all three half-rate clocks fixed this.
   Triangle and DMC still clock every CPU cycle.
3. $4017 delayed frame reset is FOUR cycles for an even numbered write,
   THREE for odd. Original flag-read sweeps at cycles 29844--29851 proved
   adjacent writes at 16 and 17 both first assert IRQ at 29848. A unit test
   covers that result and the three-cycle reassertion window.
4. Branch IRQ first polling uses `last_irq` immediately after the opcode
   fetch, before fetching its operand. A page-crossing branch additionally
   polls `last_irq` after the fixup read. Branch NMI samples the latched
   current NMI after the operand fetch ONLY; page fixup adds no NMI poll in
   the reference. Normal instructions sample current NMI at their end.
   Original JMP/branch/page-cross probes matched 57 NMI timestamps and 58
   IRQ timestamps per case, including both page-cross cases.
5. Initial PPU frame zero has the ODD hardware phase: skip pre-render dot
   339 when `frame & 1 == 0` and rendering snapshot from dot 338 is enabled.
   The old opposite parity left MMC3 IRQ markers early by an instruction
   every six frames. Inverting the initial phase gives 723/723 exact MMC3
   IRQ markers over 30 frames and 435/435 exact noise-period write markers
   over the full 660-frame Stallar input replay.

The explicit sprite CHR address presentation at dot 260 and background at
324 remains necessary; removing the sprite presentation (rise 261) produced
late IRQs and was reverted. PPU/CPU bus operation phase is also unchanged:
a palette-write diagnostic matched all six oracle frames exactly.

## Other reference discoveries

- The first `emu_run_frame` executes NO CPU cycles and produces no audio.
  Proven with an original startup-input ROM: keys are changed at frame 1
  before its first instruction runs. Nes.first_frame models that behavior.
- The reference updates one framebuffer in place, then completes a normal
  CPU instruction after the PPU wraps. Its output may include a few pixels
  from the next frame at row zero. Double buffering was substantially worse.
- CPU NMI line sampling is after the FIRST PPU tick of each CPU cycle.
  Vector selection for IRQ/BRK uses current NMI after pushing status.
- PPU status read at scanline 241 dot 1 suppresses the pending vblank set.
- Visible dot 0 must present a CHR address for MMC3 A12; doing so on the
  pre-render line too caused a bundled MMC3 diagnostic failure.
- Noise compatibility: reload the full listed period on the half-rate
  clock, giving e.g. 66 CPU cycles for period index 3 (32), unlike the NES
  hardware table's usual interpretation. Power noise shift 0x4000/timer 1.
- DMC rate index 13 is 84. Its power timer is 419 after eight reset cycles.
  DMC fetches currently align to even CPU cycles; DMC/OAM overlap is still
  modeled through nested DMA and warrants a real arbiter investigation.
- Audio output delay is 17 samples and initial clock phase is eight reset
  cycles. A 20,000-frame clock probe constrained CPU_HZ to 1789773 +/-0.06;
  the rounded 1,789,773 Hz clock prevents long-run sample-count drift.
- OAM DMA can suspend at a frame boundary; last_dma_cycle flags preserve its
  pending future. The original DMA page $40 probe reports zero controller
  data in the oracle even when strobe is high. An early-DMA variant also
  does so, while direct CPU controller/OAM reads work. This suggests a
  separate I/O peek path in reference DMA, NOT evidence about frame pausing.

## Validation and output measurements

13 unit tests plus one ABI integration test pass. Units cover mapper
banking/RAM/mirroring/header overflow, SHX page carry masking, partial DMA,
DMA parity and duration, branch NMI deferral, and frame-counter IRQ phase.

All bundled RAM-reporting diagnostics pass at 600 frames (instruction timing
at 1800). CPU reset stops at 0x81 asking for hardware Reset; cold ABI reset
cannot satisfy that request. APU diagnostics also pass after the half-rate
pulse phase correction. Screen-only sprite diagnostics have been spot checked;
sprite overflow timing final output previously matched the oracle exactly.

The 660-frame `tools/gameplay.txt` replay exercises Start, A, held directions,
and A+B and takes every bundled game beyond its title. Current audio RMSE:
Stallar ~3.53 PCM (previously ~595), File Fixers ~2.08 (previously ~12.5).
Pulse channel 2 isolated RMSE ~0.81, triangle ~0.91, isolated noise ~2.59.
The remaining differences are largely resampling/filter approximation.
Sample counts match: 483551 samples for all 660-frame game traces.

Video after the timing fixes, before the audio-only pulse correction (which
cannot change CPU/PPU timing): RFK 659/660 exact, 1 pixel total; RHDE 657/660,
3 pixels total; File Fixers 650/660, 10 pixels total; Stallar 643/660, 17 pixels
total. Every differing frame has one pixel. Almost all are row zero prefixes;
Stallar additionally differs at frames 403/404 (row 207), 414 (row 155).

## Useful diagnostics and artifacts

Everything under `.out/` is generated and ignored. Important current paths:

- `.out/input-ref-{rfk,rhde,ff,stallar}` and matching WAVs: oracle 660-frame
  replay. `.out/fixed-input-*`: native corrected CPU/PPU video; audio there
  predates the last pulse phase change. `.out/markers/pulse-odd-*`: newest
  Stallar/File Fixers/pulse2 audio and final images.
- `.out/markers/`: original probe ROMs, reference/native WAV/raw, traces,
  timing JSON and temporary variant binaries. `run-fixed-inverted` has all
  CPU/PPU corrections but pulse clocks still even; `run-pulse-odd` has the
  final correct pulse/noise phase. The production runner is rebuilt normally.
- `current-mode-measure/timing.json`: exact oracle cycle timestamps for all
  435 Stallar noise-period changes; `inverted-current-modes-events.txt`
  matched all of them. `mmc3-jmp-measure/timing.json` contains 723 IRQ markers;
  `mmc3-inverted-events.txt` matches all of them.
- `exact-noise-writes.txt` / `exact-noise-replay.nes`: replay reference noise
  write timestamps. Oracle replay vs original noise-only game RMSE 0.34;
  native corrected replay vs oracle RMSE 2.59. Native corrected noise-only
  game also RMSE 2.59, confirming its noise state no longer diverges.
- `triangle-ref.wav`, `pulse1-ref.wav`, `pulse2-ref.wav` isolate Stallar's
  channels by rerouting other length-register stores to ignored $4018.
- `.out/fixed-regression.log` and `.out/regression/results.json`: current
  full bundled diagnostic sweep (before audio-only pulse phase change).

Original assembly tools: nmi_marker_probe.s (defines RENDERING, OAM_DMA,
BRANCH_LOOP, PAGE_CROSS); irq_marker_probe.s (BRANCH_LOOP, PAGE_CROSS);
mmc3_irq_marker_probe.s with mmc3_probe.cfg; noise_phase_probe.s. Other probes
cover startup input, DAC/pulse/triangle/noise/DMC and DMA. Use ca65/ld65 and
nrom.cfg (16 KiB) or nrom32.cfg (32 KiB). These are original diagnostics.

`tools/replay_noise.py TRACE ROM` creates a cycle-accounted NROM replay,
with nested delay loops to fit an 11-second trace in 32 KiB. Options:
--registers 4011,4017 for DAC markers, --offset N for shifting writes,
--max-cycles (default 20 million).
`tools/measure_markers.py TRACE REF_WAV FRAMES OUT --radius 7` runs oracle
replays at candidate cycle offsets and reports the exact best marker times.
Inspect rmse; zero means an exact reference waveform. Cache metadata includes
trace and builder hashes. Dense IRQ markers can have tail error from previous
markers at other offsets; compare sequences against an already measured
reference cycle list instead of treating every local score as exact.

Trace feature emits WRITE cycle address value and MIX cycle p1 p2 tri noise
dmc. NES_MUTE bit8 mutes noise. NES_NOISE_SEED/TIMER are debug-only overrides.

rustfmt is absent; do not download it. Optional offline formatting:
RUSTC_BOOTSTRAP=1 rustc -Zunpretty=normal --edition 2021 file.rs > .out/pretty.rs
The required cargo build itself uses no custom flags or config.

## Next investigations

1. Exact FIR, mixer, filters and first-transient matching. Current fractional
   sinc beta .9206/radius16/offset1.334 and filters are approximations. CPU
   write timestamps and all channel states now match these game probes, so
   sample differences can be investigated without conflating CPU/APU errors.
2. The one-pixel frame-prefix differences and Stallar's three in-frame pixel
   differences. PPUMASK propagation and rendering register/OAM edge cases
   remain plausible; a palette-write probe already matches exactly.
3. DMC/OAM overlap: nested CPU::read DMC stalls can flip OAM GET parity. Need
   bus arbitration/overlap probes with audio and sprite/video timing, not
   only DMC rate tests. Controller deletion during DMC DMA also needs checks.
4. Mapper 2/3 bus-conflict/submapper behavior, MMC1 board variants, wider
   real-game input traces, and rendering-time PPUDATA/OAM behavior.
