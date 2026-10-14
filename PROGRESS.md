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
- DMC rate index 13 is 84. Its power timer is 422 after eight reset cycles.
  A sample at rate 15 empties the buffer at cycle 809 and changes DAC at 863.
  A shared Bus arbiter now handles DMC/OAM overlap; see below.
- Audio output delay is 17 samples and initial clock phase is eight reset
  cycles. A 20,000-frame clock probe constrained CPU_HZ to 1789773 +/-0.06;
  the rounded 1,789,773 Hz clock prevents long-run sample-count drift.
- OAM DMA can suspend at a frame boundary; last_dma_cycle flags preserve its
  pending CPU operation. This is now implemented in Bus, not Cpu futures.
- Triangle timers clock BEFORE coincident frame-counter gate updates. A
  2400-frame File Fixers replay exposed a permanent phase error at 27.9 sec;
  moving the timer ahead of linear/length clocks fixes it (full RMSE 121 ->
  2.04). Small original start/stop edge probes and a unit protect this.
- Sprite evaluation updates the shared OAMADDR byte after each even dot.
  Disabling rendering mid-evaluation preserves an unaligned address, which
  rotates the next DMA if software omits $2003. RFK's software reboot while
  evaluating at line 205 dot 107 left OAMADDR 75. Tracking that fixed its
  one-frame sprite disappearance mismatch in a 5000-frame varied replay.

## Verified DMA and controller architecture

CycleOp holds a pending CPU Read/Write. Bus.execute ticks devices FIRST,
then arbitrates the same cycle; it returns false while DMA owns the bus and
keeps the CPU operation pending. Fields track OAM page/index/latch/data,
DMC pending time/stage, and the halted CPU address. DMC GET wins even cycles;
OAM GET/PUT can proceed while DMC does halt/dummy/alignment. A stolen OAM
GET creates one PUT alignment cycle, normally adding two CPU cycles. Ten
original tail-overlap cases also match their one/three-cycle extensions.

First DMC enable schedules halt at write+4 for even writes, +3 odd. A buffer
empty event schedules halt on the current PUT or following PUT. The new
422 power timer plus same-cycle arbitration matches initial load marker 55,
OAM overlap marker 983, and every measured DMA tail marker exactly.

DMA external addresses access RAM/PPU/cart normally; internal $4000-$401f
floats. Internal 2A03 I/O decode combines high bits from the halted CPU
address with low five DMA address bits. Only resulting $4015/16/17 activates
status/controllers. Actual GET updates CPU open bus; repeated halt/alignment
reads preserve its old value while retaining peripheral effects. Original
page-$40 OAM probes produce $40 from the prior DMA write, or $ff after an
interleaved DMC GET, matching the reference. Do not restore the older peek
hypothesis or nested CPU DMA implementation.

Controller shifting occurs once per contiguous /OE assertion. Repeated DMA
halt reads return the latched bit; a DMC cart read can release /OE and delete
one bit. A DMC shadow read selecting the same controller keeps /OE asserted
and corrupts fetched audio data with controller bit zero. Both original
controller-DMC probes match video/input and the shadow case audio. Normal
CPU intervening accesses and OAM PUT release /OE. Tests protect these cases.


## Validation and output measurements

24 unit tests plus one ABI integration test pass (latest changes included). Units cover mapper
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
RHDE ~0.94 and RFK ~0.45. Native/wasm Stallar framebuffer/audio hashes
match exactly: 455016102 / -520129133.
The remaining differences are largely resampling/filter approximation.
Sample counts match: 483551 samples for all 660-frame game traces.

Latest video: all 2640 frames of the four 660-frame game replays match the
oracle byte-for-byte. Two additional findings removed the last pixels:

- The callback publishes the pixel prepared for the next PPU dot. The new
  Ppu::publish_next_pixel exposes it without advancing time, sprite shifters,
  or status flags. Seven original palette-write boundary variants matched
  all six frames each, including writes on the very last CPU bus cycle.
- At column 255 the reference already uses the following scanline's sprite
  count, while the pixel shift data still belongs to the current scanline.
  Limit output slots there to min(current_count, eval_pos/4). Independent
  solid-sprite probes showed bottom-right pixels missing only when the
  next-line count drops; adding an unrelated sprite on the left restores
  the first slot's bottom-right pixel. Swapping OAM order proved it is a
  count limit rather than a per-sprite Y check or a global empty-line test.
  This fixed Stallar frames 403/404/414. Units protect both behaviors.

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

1. Exact FIR, mixer and filters. Current fractional
   sinc beta .9206/radius16/offset1.334 and filters are approximations. CPU
   write timestamps and all channel states now match these game probes, so
   sample differences can be investigated without conflating CPU/APU errors.
2. Broader game replays and rendering register/OAM edge cases. The bundled
   660-frame videos now match exactly, but that is not exhaustive validation.
3. Rendering-time OAM DMA still writes the raw OAM array rather than the
   PPU register path. Probe its write blocking/address increments and wider
   DMA peripheral interactions. DMC/OAM arbiter and controller deletion are
   now validated by original probes and bundled rate tests.
4. Mapper 2/3 bus-conflict/submapper behavior, MMC1 board variants, wider
   real-game input traces, and rendering-time PPUDATA/OAM behavior.

## Current resampler investigation

`.out/resampler/random-dac.nes` plays 800 original randomly spaced 0/96 DAC
transitions; trace in random-dac.txt, reference WAV and native raw nearby.
Current native RMSE ~5.23 PCM; old fast periodic DAC probe RMSE ~11.7.
`extract.py` removes HP90/440 and differentiates to obtain normalized impulse
observations. IMPORTANT: dequantize PCM with +0.5*sign(pcm) before inversion;
otherwise truncation biases inferred DC gain (~0.99964 vs true ~1.0).
`impulses.npz` contains coordinates, observed impulses, events and PCM.
Blackman point-sinc fits beta .92068, radius16.004, offset -1.30904 relative
to the CPU WRITE timestamp, LP alpha .86059/gain1.0029, but residual is still
large. Box-integrated sinc models were worse. Per-event phase fitting shows
~0.002-sample scatter, partially consistent with a 256-phase quantization;
no resampler implementation change has been made. Sources fit_models.py and
fit_phases.py are exploratory, not validated DSP replacements.

New framebuffer artifacts: `.out/published-{rfk,rhde,ff,stallar}` match the
first three completely; Stallar there predates the column-255 count fix.
`.out/edge-fixed-stallar` matches all 660 frames. `run-edge-fixed` is the
latest non-trace native binary; the standard runner is rebuilt as needed.
Sprite probes use sprite_edge_probe.s + nrom_chr.cfg; defines OVERLAP and
COUNT_DROP select the corroborating OAM-order variants.

The DMA reference documentation now explains the old page-$40 probe:
2A03 internal register decode takes high address bits from the HALTED CPU
address, and low five bits from the DMA bus. An OAM read of $4016 while the
CPU is halted at ROM does not activate the controller. This is hardware
address gating, not proof of a side-effect-free DMA peek. A proper arbiter
should preserve the halted CPU address, overlap DMC halt/dummy/alignment
with OAM transfer cycles, and keep GET/PUT parity. See spec/docs/.../DMA.md.

## Latest longer gameplay and startup audio validation

`tools/extended-gameplay.txt` extends the original replay to 2400 frames.
All 4800 video frames of Stallar/File Fixers match, with 1760348 samples.
Current mixed audio RMSE is Stallar 3.898 and File Fixers 2.027 after the
triangle gate fix and startup impulse correction. Old FF error grew above
200 PCM after 28 seconds; that state divergence is resolved.

`tools/varied-gameplay.txt` uses seed 0xc02a03, 136 changes over 5000 frames,
including Select, Start, one-frame taps, held inputs, opposing directions,
and all buttons. All 20000 video frames match after OAMADDR tracking (other
three native directories still predate that PPU-only change, so recheck).
All four have 3668206 samples. RMSE RFK .000738, RHDE 1.217, FF 2.039,
Stallar 3.858. References .out/varied-ref-{rfk,rhde,ff,stallar}, WAVs adjacent;
native .out/varied-native-* except newest RFK .out/varied-rfk-oamaddr.

The fixed startup phase uses a calibrated cumulative FIR impulse after the
17-sample discard, derived from an original no-write power audio ROM and
inversion of the known analog filters. First 16 samples match exactly across
six probes/games; no per-ROM behavior. HP/LP coefficients now use exact
exp(-2*pi*f/44100) values. Power-only RMSE .012, RFK660 .002, random DAC
5.153 (old 5.227). .out/audio-before-startup.rs is the old implementation;
.out/run-tri-fixed has corrected triangle timing but old startup kernel.

Minimal triangle-edge traces .out/triangle-edge/phase0.txt and ROM place a
period-84 reload at quarter-clock 14935. Old native RMSE after sample400 was
8.125, corrected .0199. Condensed FF triangle write replay in
.out/ff-tri-condensed.txt must retain the first length write AFTER enable;
removing all identical $400b values incorrectly leaves it disabled.

Offline exploratory FIR fitting: .out/resampler/fit_spline*.py recovers a
single continuous impulse response with cubic splines from 800 random DAC
transitions (half held out). 1/64 spacing reduces inverse-filter residual to
~2.36 PCM versus ~13.65 for point sinc, but no normal FIR implementation
change yet. .out/resampler/spline.npz is exploratory. Quantized phase256
partly explains per-event delay scatter; fit_phases.py reports residual std
.00157 versus .00195 continuous. Need validated unseen probes/dense game
replays before replacing the normal kernel.
