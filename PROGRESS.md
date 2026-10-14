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
- `src/audio.rs`: nonlinear mixer, calibrated polyphase FIR
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

26 unit tests plus one ABI integration test pass (latest changes included). Units cover mapper
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
3. Wider DMA peripheral interactions and rendering-time OAM side effects.
   The reference directly writes OAM during DMA even when rendering is on;
   a direct CPU $2004 write is blocked. Original render_dma_probe.s proves
   both paths (DIRECT_WRITE variant). Do not unify those paths blindly.
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
and all buttons. All 20000 video frames match after OAMADDR tracking; the other three
games were also rerun after that change and all 15000 frames matched.
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

## Latest calibrated FIR and frame-mode correction

Normal FIR is now a generic cubic B-spline reconstruction filter in
src/audio_kernel.rs, spacing 1/256 sample, support +/-16. Polyphase kernels
are computed at startup and normalized; their values are independent of
cartridge data. The timing quantizer floors the absolute CPU WRITE phase to
1/256 sample before removing eight reset cycles. In Resampler the observed
level changes on the NEXT CPU tick, so add 7 reset-cycle fractions to the
local clock before quantization. This was derived from the pattern of
per-event phase residuals, not a game-specific adjustment.

`tools/calibrate_audio.py WAV TRACE POWER_WAV OUTPUT` reproduces every
current coefficient exactly. Example inputs are .out/resampler/random-dac-
ref.wav, random-dac.txt, power-ref.wav. It first recovers a spline from
inverse-filtered impulses, holding out half the phases, then refines the
coefficients in PCM space with DC constraints and small smoothing. This
avoids amplifying integer PCM quantization noise in the inverse filters.
The coefficients were validated against a fresh 700-write varied-DAC ROM
(seed 0xdac2c02) and game audio, independent of the original DAC pattern.

Current .out/pcm-fit-* measurements: random DAC RMSE .1005, fresh DAC .1504,
File Fixers2400 .1163, Stallar2400 .2099, RHDE660 .0442. Dense game peaks are
2 PCM; fresh DAC peak3, original peak4. Coefficient calibration is generic;
no output samples or game state are stored in the emulator. Native and wasm
File Fixers2400 hashes match: framebuffer445537872 / audio-2041901648,
1760348 samples at44100. Stallar2400 native hashes1283754735/-1062086199.
Old analytic FIR backup .out/audio-with-startup.rs; old calibrated continuous
kernel .out/audio-kernel-continuous.rs; quantized128 and quantized256 backups
nearby. .out/run-audio-trials.py runs the five current reference comparisons.

Frame sequencer $4017 mode bit7 takes effect on delayed reset; IRQ inhibition
bit6 takes effect immediately. Old code changed both immediately, missing
IRQ flags when switching to mode5 just before the old mode4 edge. Added
frame_active_five, updated on delayed reset. frame_mode_probe.s tested modes
0/64/128/192 at writes29844/46/48/50/52; mode128 at48/50 previously mismatched
and now matches. Unit protects mode delay versus immediate inhibition.

Original oam_rotation_probe.py disables rendering at line61dot70, writes
$2004, and encodes readback of attribute byte2. Reference/current palette47,
old native15; this independently proves the shared OAMADDR fix. RFK varied
replay disabled at line205dot107 retaining75 and rotating its next DMA.

All30 RAM-reporting bundled diagnostics still pass after these changes
(reset ROM remains at129 asking for hardware reset). A new oracle final-
frame/audio sweep is running, output .out/diagnostic-ref/results.json.
This may expose reference-versus-hardware diagnostic differences; inspect
its results rather than assuming passing RAM status means exact output.

## Resolved investigation: MMC3 background-$1000 IRQ timing

The oracle diagnostic sweep completed: all30 final images matched EXCEPT
mmc3_test_2__4-scanline_timing. Oracle fails #9: "Scanline 0 IRQ should occur
sooner when $2000=$10". Native passes. Its final image differs987pixels,
audio RMSE207.88, native samplecount+1. This needs compatibility investigation;
do not special-case the test or assume hardware pass means reference match.
Reference PNG .out/mmc3-scan-ref.png versus native .out/mmc3-scan-native.png.
Whole PRG disassembly .out/mmc3-scan-disassembly.s. Test9 starts LE641,
setsctrl$10 and latch0 via LE408, then samplesIRQtiming LE41C. Trace
.out/mmc3_test_2__4-scanline_timing-trace.txt includes PPU_WRITE and APU writes;
MASK was not retained by this filtered trace. CTRL$10 writecycle6251572
(frame209line241dot66) is probably test9; laternative continuesalltests
whereoraclestopsat9.

Updated original mmc3_irq_marker_probe.s supports CTRL(default8) and LATCH
(default8); settingCTRL16/LATCH0 reproduces mismatch without the bundledtest:
.out/mmc3-bg1000/probe.nes, probe-ref.wav (30frames), native/audio.raw
RMSE191.3 peak761. Allsprite-$1000/bg-$0000 markers previously matchedexactly,
so likely the background A12 presentation/filter or reversal-case edge.
Current PPU explicitly presents backgroundCHR at dot324 (and visibledot4
of each8); nativeactualreadoccurs325 (visible5). SpriteCHRpresent260 remains
correct for prior723referenceIRQtimestamps and games. Visible dot0 CHR
presentation is also currentlyrequiredbyearlierdiagnosticexperiments.
Do not change those globally without regression ofbothpatterns.

A timing measurement job is running in exec session24782: builds trace,
runs probe30 retainingWRITE rows .out/mmc3-bg1000/writes.txt, then original
DAC-only replays of allmarkers beforeCPU120000 for offsets -2..2. Each
candidateoracleWAV (5frames) is comparedtooriginalref first2850samples;
zero RMS means those nativeCPUtimestamps plusoffset exactly matchreference.
See its output next to determine the direction/size of the edge error.

The two APU lengthdiagnostic audio discrepancies were ultrasonic triangle
periods0/1. Reference holdssequencer/DAC15 while gates/counters stilloperate.
Addedperiod>1 to triangle sequenceadvance. Parameterizedtriangle_probe.s
PERIOD0..4 proves cutoff: 0/1 RMSE.012peak1, 2..4 RMSE.35-.38peak1.
All26units+ABIpass and wasm built with thisfix. Lengthtable600 audio RMSE
.01349peak1, lengthtiming600 .03021peak1 (previous55.3/42.9). Outputs
.out/ultra-fixed-apu_test__*. Initialdiagnosticresults.json stillcontainsold
largeaudioerrors; referenceWAVs remainvalid. Everyotherdiagnostic audio
was~.01-.017RMSE andpeak1 exceptMMC3 failure. Nextregularnative buildshould
remove trace feature; currentbinarymaybetrace fromrunningtimingjob.

## Current verified state after resolving background MMC3 edges

The background-$1000 issue is resolved by three general PPU changes:
1. Remove the artificial early background CHR presentation at dot4/324.
2. Preserve A12 low across visible dot0 when background uses $1000. Keep
   the old dot0 low presentation for a $0000 background.
3. Read background pattern data at dots5/7, but present its cartridge address
   at dots6/8 (prefetch326/328/etc). Sprite early presentation260 remains.

With only the first change the oracle MMC3 scanline test already matched,
but repeated IRQ markers still differed. Removing upper-table dot0 restored
all1302 IRQs in CTRL16/LATCH8. A generic coordinate-descent DAC marker fitter,
tools/refine_dac_markers.py, inferred155 timestamps +3CPU and1147 unchanged.
An original DAC-only replay of that inferred list produced EXACT oracle audio
(RMSE0), proving those timestamps independently of the approximate FIR.
Delaying the background address by one dot then made native all1302 cycle/
value pairs equal that exact list. .out/mmc3-bg1000/delayed-writes8.txt equals
refined8.txt at every4011 write; delayed8/audio.raw RMSE.438peak4 (DSP-only).

A PPU unit protects the cartridge IRQ edge on dot6, after data fetch5, and
preserving low337->6 across scanline boundary. All27unit+ABItests pass. The
required wasm is built with this implementation (149KiB, no imports).
Eight original marker configurations CTRL0/8/16/24/32/40/48/56 LATCH8 allmatch
samplecounts and have RMSE.01-.44, peaks<=4. This covers both pattern tables
and8x8/8x16 sprite evaluation/fetch configurations, with background enabled.
References/native .out/mmc3-layout-{ctrl}-ref.wav and matching-native dirs.

All20000 video frames in the four varied5000-game replays match AGAIN after
all new APU/PPU/FIR changes. Native .out/varied-final-{rfk,rhde,ff,stallar}.
Audio3668206samples each, RMSE RFK.000738, RHDE.073714, FF.118370,
Stallar.202337. Peaks RFK1, RHDE2, FF4, Stallar10. Rare larger Stallar sample
errors deserve investigation; no sustained channel state divergence observed.

Latest bundled sweep .out/final-current-regression.log and
.out/regression/results.json. Expected exceptional statuses: CPUreset129
(waiting for hardware Reset, matchingreference) andMMC3scanline9 (matching
referencefail9). Everyother diagnosticstatus0. The new MMC3 final image is
byte-exact and audioRMSE.02975peak1, samplecount439524 matchingreference.
Latest sourcecomments/unit may not be in the previous supervisor checkpoint;
all sourcefiles are present and compiled in the current wasm.

Mapper2 original bus-conflict probe uxrom_conflict_probe.s/uxrom_probe.cfg
hasinitialROMbyte0 at writeaddress8000 and selectsbank2($22). Reference and
native output34 forlegacyheader andNES2submappers0/1/2. Reference ignores
bus conflicts evenwhenNES2submapper2 requests them; current behavior matches.
.out/uxrom/sub{-1,0,1,2}.nes andref/nativeoutputs retainproof. Do not introduce
hardware bus conflicts based solely on the NESdev docs withoutoracleprobes.

Current next work: original MMC1 board/outer-bank/RAM probes and CNROM,
rendering/PPUDATA/OAM edge cases, more varied inputs, and exact normal FIR/
phase/mixer arithmetic. The task remains active indefinitely; no conclusion.

## MMC1 expansion and rendering address pins verified

MMC1 PRG outer bank, SOROM/SXROM RAM banking, and SNROM RAM disable now
follow the active PPU address in 4 KiB CHR mode. Twenty-five original board
probes match final oracle images. Expansion pins are used with 8 KiB CHR;
larger CHR-ROM ignores them in the reference. SNROM's CHR bit4 RAM disable
requires CHR-RAM, <=256 KiB PRG and 8 KiB RAM. CPU $2006 writes select the
pins immediately, even without a subsequent $2007 access.

Rendering fetches present MMC1 addresses one dot after their data access;
background pattern pins rise at6/8, sprite pins retain early260. End-of-line
nametable fetches337/339 present immediately. MMC3-only synthetic low edges
256/320 must not change MMC1's pins. PPU cart_addr_pending implements the
ordinary fetch delay without delaying CPU port accesses.

tools/mmc1_ram_phase_probe.s streams selected RAM values into $4011 while
rendering. Eight PPU layouts times eleven CPU alignments (88 original ROMs,
20 frames each) all match samplecounts, max audio RMSE .425911, peak6.
Results .out/mmc1-phase/sweep/results.json. Generic DAC fitting isolated
161 wrong reads at dot337; flipping the inferred values there and replaying
their exact CPU write timestamps through an original DAC-only ROM produced
oracle RMSE0, proving the pin timing independently of the approximate FIR.
The verified source has NO CPU instruction snapshot of MMC1 pins.

RHDE's 5000-frame varied-input replay has now been rerun after the delayed
MMC1 fetch changes: every frame remains exact, audio RMSE .073714 peak2.
The MMC1 pin probe's wasm framebuffer/audio hashes equal the native runner,
13925 samples for20 calls. There are no host imports.

31 unit tests plus the ABI test pass; required wasm was rebuilt. New tests
protect board expansion conditions, SNROM disable, and delayed ordinary /
immediate end-of-line fetch pins. RHDE5000 after the PPU pin
delay matches every image and retains its previous audio residual.

## CNROM bus conflicts fixed

CNROM, unlike the reference UxROM, ANDs bank writes with the ROM byte at the
write address. Original tools/cnrom_conflict_probe.s varies that byte through
0/1/2/3/255 with CPU value3, legacy and NES2 submapper0/1/2 headers. All20
images now match; before the fix the first12 selected incorrect CHR bank3.
The reference applies conflicts even to NES2 submapper1. Cart::write now
applies this AND only for mapper3. Unit protects the address-specific ROM
drive; existing CNROM bank test now uses a $FF ROM byte.

The same original ROM has a STREAM mode repeatedly switching banks and
reading CHR into $4011. Forty calls produce28601 samples, RMSE .328798,
peak3. Wasm and native hashes match. 32 units+ABI pass; required wasm built.
Artifacts .out/cnrom/, tools/cnrom_probe.cfg. Explicit NES2 CHR-RAM shifts
0/1/4/5/6/7/8/9 all behave as at least8KiB in the reference (no $0000->$1000
mirror), matching our allocation. Original tools/chr_ram_size_probe.s and
.out/chr-size retain evidence. Continue probing instead of changing sizes.

Stallar5000 audio has only seven samples with absolute residual>=5:852597
(+5),852598(-10),852599(+5),1499398(+7),1763998(-7),1848826(+6),1848827(+8).
Times19.3333/34.0000/40.0000/41.9235 seconds. Investigate isolated event/
quantization rounding rather than assuming persistent APU state divergence.

## Audio phase boundary rule identified with original DAC probes

Stallar's rare residuals are phase quantization, not APU sequencing. An
isolated DAC-only ROM at the same CPU cycles reproduces the errors. A new
tools/dac_phase_probe.py uses modular inversion of14700/596591 to place
256 isolated impulses immediately below all256 phase boundaries. Reference
selects the preceding sample's phase255 at exact phase0; boundaries143/173
round up, all253 other cases retain the lower phase. This matches rounding
the scaled fractional phase to f32 before the8bit table lookup. Native
formerly floored f64 everywhere. Original artifacts .out/dac-phases/near.*,
.out/dac-boundaries.*; analysis .out/dac-phases/phase-analysis.txt.

Resampler now computes source phase directly from its integer accumulator,
rounds the scaled phase through f32, and preserves the reference's lower
side at whole samples. Unit covers zero, two rounded-up boundaries, and
two boundaries that must stay below. All33unit tests pass. Four5000-game
audio regression is running (.out/phase-boundary-*); required wasm must be
rebuilt after confirming it. Backup .out/audio-before-phase-boundaries.rs.

Further MMC1 probes: SEROM submapper5 and allother tested0..7 act like
ordinary banked MMC1 in the reference; tools/mmc1_serom_probe.s/.cfg tests
this without assumptions. All9 final images match. SNROM streaming disable
CHR1=16/RAM8KiB across8PPU layouts matches audio timing/samplecounts; RMS
.0147-.991 peaks1..5 (large17->96mixsteps amplify generic FIR residual).
Artifacts .out/mmc1-serom and .out/mmc1-snrom.

The phase correction is validated: near-boundary256probe RMS .035470,peak4
(previous .093992,peak77); six isolated boundary impulses RMS .014319,peak4
(previous .184466,peak77). All four varied5000 game samplecounts3668206 and
final images match; audio RFK .000738peak1, RHDE .073714peak2, FF .118294peak1
(previouspeak4), Stallar .201956peak3 (previouspeak10). No residual>=5
remains in any game. tools/audio_phase.py shares the integer-rational/f32
phase adjustment with calibration and inference tools. New wasm build and
Stallar2600 native/wasm hash check are in progress.

Latest wasm was built after33unit+ABI pass. Stallar2600 native andwasm
hashes match (frame -861571252,audio376821155,1907107samples,44100Hz).
Timed OAMADDR8 writes before/during sprite-fetch across12CPU offsets do
not cause row-copy corruption in the reference; native matches palette17
in allcases. tools/oam_corruption_probe.py and .out/oam-corrupt retainproof.

NES2 small-PRG original ROMs (1/2/4/8KiB exponent sizing) are rejected by
the reference;16KiB is accepted. Our loader previously accepted smaller
images, then could divide byzero reading the reset vector. Cart::load now
rejects PRG<16KiB. UxROM fixed-window arithmetic rewritten n-2+slot%2 to
avoid a debug underflow with a single16KiB bank. Newunit covers rejection
and single-bank reset-vector mapping. New34unit/build checks running.

Bundled diagnostics after all current fixes: all30 final images andall30
audio samplecounts matchreference. Maximum audio RMS .030205 (APU length
timing); peaks1 everywhere except PPU NMI-on timing2. Detailed current
results .out/phase-final-diagnostic-comparison.json, native status output
.out/phase-final-diagnostics.log. Required wasm built with34unit tests.
32 original immediate8B/AB opcode probes varyingA/operand andcapturingA/P
allmatch full8bit palette+emphasis output. tools/unofficial_probe.s and
.out/unofficial/results.json retainproof.

## Active new discrepancy: DMC load colliding with PPUDATA instruction

Original tools/ppu_dmc_probe.s initializes CHR17/34/51/etc, starts DMC,
then reads2007 immediately. Sixteen parity/NOP/prime combinations:14match,
two fail (PARITY0,EXTRA_NOPS0,PRIME0/1). Native returns0/17 vsreference17/34
with audio RMS131/117. Allothercasesmatchaudio<.09. In failingcase enable
write30859odd, DMC_HALT30862 heldCPUoperand-highaddress8063, DMC_GET30864,
actualPPUDATAread30866. Reference behaves as oneextraPPUDATAread eventhough
DMA halts the preceding operand fetch. PARITY1 enables30862even, halts
2007at30866, rawholdreads30866/30867,GET30868,realread30869; native/reference
both return34. Do not blindly changeDMAgettiming or rawholdreads.

STRICT proof: take failing nativeMIX trace, change finalDAC0->17 andevery
subsequent DMC level+17; build original DAC-only replay at ALLunchanged CPU
write timestamps. Oracle replay vsfailing original produces RMSE0,peak0.
LastCPU4011write30882 is alreadycorrect; shiftingit±1 producesRMS~4.97.
Artifacts .out/ppu-dmc/full-replay0.*,p0-full.trace,results.json,p?-n?-b?.trace.
Thus fix side effects, not CPU stall length. CPU trace now optionally emits
DMC_HALT, DMC_GET and PPU_READ for this investigation. Controller variant
READ_REGISTER4016 withkeysAA is running, to determine whether the extra
read is generic or specific to PPUDATA. Native binary is being rebuilt
withouttrace by that job. Source has no tentativeDMAbehavior changes yet.

CNROM reference rejects CHR-RAM images (16/32KiBPRG, CHRRAMheader shifts
0/6/7/8 allreject) andaccepts8KiBCHRROM. Loader now requiresCHRROMformapper3.
Unit added; runfulltests/build after resolving the current investigation.

## Deferred DMC data-read side effects fixed and validated

Additional probes coverBIT,absX(with/withoutcross),indirectY,APU4015status,
controllerfullbyte, andPPUSTATUSnearvblank. Missing read is specific to DMA
interrupting the LASTaddress formation read, including zero-page indirect
pointer-high fetch. Page-cross dummy reads consume it before the data port
and must NOT add an extra finalread. IfDMAhalts the final4015itself, its
floatingbit5 retains pre-DMCGET bus whilecontrollerupperbits seeDMAFF.
Generalopen-bus4015testswithpagecrossdummyPPUbus0/32/255provebit5 otherwise
followsregularCPUopenbus; do not globallyzeroit.

Bus now recordslastDmcGET andpreGETopen; CPU.finished_address marks a read
repeat when the address was fetched immediatelyafterthatGET. The nextreal
CPUread performs the extraaccess (and releasespadOE betweenitandactualread);
pagecrossdummy consumesit, writesdiscardit. NoCPUcyclesadded. Final4015
held-byDMA usespreGETfloatingbit5 forits resumedread only. Backupbeforefix
.out/cpu-before-deferred-dmc-read.rs. All60originalcollisionprobesmatchvideo
now (.out/ppu-dmc/fixed-results.json), maximumaudioRMS .08823. Includes11
PPUSTATUSvblankalignments, allmemory-addressingvariants andcontrollerbits.

36unit+ABI pass; wasm rebuilt. NewCPUunit coversabsolutePPUDATA/controller
andAPUstatus,addressindirectpointerhigh,andpagecrossnegativecase. Current
bundleddiagnostic rerun .out/deferred-dmc-diagnostics.log completed; compare
itsimages/audio next. Dense loopingDMC+PPUDATAstream5alignmentprobes running
(.out/ppu-dmc-stream); theyexercise reloadcollisions, notonlyfirstload.
Original tools/ppu_dmc_stream_probe.s/tools/nrom32_chr.cfg.

Bundled diagnostics rechecked afterdeferredDMCfix: all30finalimages/counts
matchagain, maxRMS .030205 peak2. Results
.out/deferred-dmc-diagnostic-comparison.json. Wasmcheckedfailinginitialload
probe now hashesFB52514245/audio1665564754,2184samples.

## Active audio investigations after dense DMC/PPUDATA stream

Dense stream .out/ppu-dmc-stream/n{0..4}: all87304samplecounts match120frames,
RMS n0=8.681,n1=.309,n2=6.714,n3=9.281,n4=8.678, peaks136/2/137/137/137.
These are NOT persistentPPUreaderrors: forty actualreloadisolatedprobes
allmatchfinalvideo; onlytwohaveaudioerrorsfromDMCstop timing (below).
StreamfirsterrorPCM2674 coincideswithDMCstep108579 ANDCPU4011write108579.
NativeMIX108579DAC8 (previousCPUvalue6+2), writeDAC7 samecycle, MIX1085807.
Reference likelypreservesincrementagainstcoincidentnewDACwrite (9), but
verifywithoriginaltools/dmc_dac_write_probe.py beforechanging APU. Twenty-
fourmatrixprobesold64,new0/64/126/127,sample00/FF,CPUwrite862/863/864running
.out/dmc-dac-write/results.json. FirstactiveDMCoutputclock863. Single-byte
nonlooping sample avoidsreloadDMA; generatedCPUdelay subtractsinitial3
DMAcycles. PriorstrictDACreplayofnormalDMCsteps provednormalstep timing
correct; do NOTgloballyshifttheDMCtimer orresamplingphase.

CorrectreloadprobesRELOAD_DELAY149..153,EXTRA_NOPS0..3,PARITY0/1 (40cases)
.out/ppu-dmc-reload-real/results.json: allvideomatch. Delay149parity0nops0
audioRMS7.077, nops1RMS99.999; allothers<1. DMCstopnearbufferempty31625 is
likelyaborted/unexpectedDMA (documentedinDMA.md). Stop2CPUbeforeempty adds
oneabortedhalt; stopattheemptycyclemayretainfullpendingreload despite
left0. InvestigatepreciseCPUwrite timesandsamplebuffercontents beforefix.
EarlierRELOAD_DELAY34..38 probes30cases were tooearly becausewriting4010
late preservesdefaulttimer remainder; theydoNOTtestreloadcollision. Actual
firstbufferempty inppu_dmc_probe.s is31625, not31049. Keepthiscorrection.

Currentnativebinary isdefaulttraceOFF. Trace forstreamn0 is
.out/ppu-dmc-stream/n0.trace (120framesincludesMIX/WRITE/DMC_HALT/GET/PPU_READ).
SourcecontainsvalidateddeferredDMCfix and36units; noDMC DAC-write trialyet.

CorrectionafterfirstDMCclockmatrix: ALL24DACwrite862/863/864probesmatch
audio RMS<.065peak1. Do NOTchangeDMC4011write ordering basedonstream alone.
A genericFIRfit ofthefirst stream residual proves reference hasone extra
PPUDATAincrement for the remainderofthat32readblock: add1toallnativeDAC
levels starting108596 through108909 gives predictedresidual86/133/110/108/
92/87/71/79/-9/-62/-48, observed -86/-134/-109/-108/-92/-88/-71/-79/9/63/48.
So stream's audioerror is a missingPPUread, NOTDMCcounter/write ordering.
DMC_HALTat108580heldDEXopcode8034 afterCPU4011write108579; GET108582, next
PPUDATAread108591. Previously modeledextraREADonlyduringaddressformation
is insufficient here. AnotherlikelymissingreadoccurswithDMC_HALT109011
heldSTA4011highoperand8033,GET109014, CPUwrites109016; nextLDA2007later.

OriginalSTORE_TARGETvariant ofppu_dmc_probe.s doesLDA2007,STA(target),
LDA2007 nearreload31625 toidentify whetherCPUstores preserve/repeat the
previousdataportread. Targets4011/6000/2000/4000,3NOPalignments,2parities,
24cases running .out/ppu-dmc-store/results.json. KeepcurrentCPU fix until
thisprovidesbettergeneralmodel. NoAPUwrite-order changehasbeenmade.

## Explicit DMC stop DMA glitches fixed

New tools/dmc_stop_probe.py isolates stop cycles803..812 aroundbufferempty
809 with a DAC marker20clocks later. Ref/native initiallydisagree forstop
806/807(RMS3.6),808/809(RMS~99); allothercasesmatch. Apu now schedules
one-clock abortedreload atemptywhenstopped2/3CPUearly, andretainsfullDMA
whenstopped1CPUearly orontheemptyclock. Bus mergesabortedhaltwithOAM or
cancelsitonCPUwrite; retainedfullDMA ignoresthezero-reader-lengthcancel.
APU.supplyallows retainedbytewithoutunderflowing/restartingdisabledreader.
All10originalstopprobesnowRMS<.071,peak1 (.out/dmc-stop/*-fixed).
37units+ABIpass andwasmrebuilt. NewCPUunit'sactualmarker timestamps828for
stop807 and832forstop809 matchreference. Backups
.out/{apu,cpu}-before-stop-dma.rs. Implicitone-byte endingbugsarenotyet
implemented; investigatewithoriginalprobesnext.

## Late PPUDATA/DMC discrepancy narrower than previous hypotheses

Originaltools/dmc_late_read_probe.py reproducesfirststreamerrorinvideo:
keepDMCsetup30777/30783/30789/30795;delay with723DMAclocks;primePPUaddr6;
reads108562->6,108575->7,write7at108579, DMCget108582, finalread108591->8
native,9reference. CHRRAM/CHRROMbothfail identically. DifferentDMC length
17/33bytesdoesnotchangeit. ShiftCHRdata so lastread/writevalue1,0,8..23:
ALLmatch. Thefailure requireslastCPUbus value7, notgeneraldataoffset!
Changingreadportto200F(whichstillmapsPPUDATA)avoidsit. STORE6001/4000/2003
insteadof4011stillfailsifvalue7. STY withY0/1/6/8 matches;Y7fails, even
whenwritingRAM6001 andPpuBusstill7. Thus CPU's data bus, notPPUbus, controls
thisextra read. Thismayreflectreference'scached peripheral callback/dummy
read using thebus value asregisterindex. DO NOTaddROM-specificmagic;
measurea generalbus rule. 

Snapshot --intervening-read replaces3clockdelaywithBIT00afterlastPPUDATA
read(beforewrite7), tocheckwhetherordinarydatareads clearcachedPPUread.
Status-read snapshotsatDMCedge58467 duringvblank inspectbus values0..8
forphantomPPUSTATUS(2)/OAMDATA(4)/PPUDATA(7)accesses. Resultsongoingin
.out/ppu-dmc-late; toolsupports --edge-cycle,--after-port,--store-y,--read-port.
Mirrored200FY15also tested tocheckcanonical-address filtering. NOP/branch/
opcodefetchmaynotreplace theprevious meaningful dataperipheral callback.
24early-storeprobes .out/ppu-dmc-store/results.json allmatchbecauseCPUbus
valuewas0, not7. Full37sourcebuildcontainsNOtentativelatePPUbusfixyet.

## General DMC register selector replaces address-phase approximation

The late discrepancy isresolvedbyonegeneralbus rule, notROM/data-specific
magic. AtDMCGET recordthepreGETCPU busbyteas a selector. The nextnormal
CPU I/O read consumesit; ifitscanonicalregisternumbermatches, performone
extra read (controllerOE releasedbetweenextra/real). RAM/ROM reads and
writes leaveitpending. PPUmirrors consumeitwithoutrepeat. CanonicalPPU
2000..2007useindices0..7; APUREG4000..401Fuse0..31. Capture comesfrom
existingdmc_open_before_get. Bus.dmc_register_selector replacesold
dmc_read_after_address andCPU.finished_address (bothremoved). NoCPUclocks
added. Existing4015floatingbit5preGET rule stillnecessary forheldstatus.

Proof: lateCPUbus7 ->next2007repeats; bus2 ->next2002repeats/clearsvblank,
evenwhenVBLsetAFTERDMA andbeforethelaterCPUread. Bus22->later4016 deletes
onebit;bus21->later4015clearsframeIRQ. NonmatchingfirstI/Oreads consume
selector; RAM BIT0/6000doesnot. PPUMirror200F/2207/3FFFdoesnotrepeat. Indexed
2000+X7mustNOTrepeat becausepreGEToperandlow0 differsreg7; oldaddress-phase
approximationdidrepeat. Originalselectmatrixandfull snapshotvariations
confirmallofthis. Backup .out/cpu-before-dmc-register-selector.rs.

215originalcollisionROMs nowallmatchfinalimages (.out/dmc-selector-
regression.json), maximumaudioRMS .16013. DenseDMCstream120frames5layouts
all87304samples, RMS .320627/.309243/.299936/.289501/.282816, peak2each
(previous4layoutpeaks136/137 andRMS6.7..9.3).
.out/ppu-dmc-stream/selector-results.json. 38units pass; wasmrebuilt.
NewunitcoversselectorretentionacrossRAM andconsumptionbyI/O; previousCPU
unitnowalso testsindexednomatchandmirroredportnegativecases. FullABI+
wasmstreamhashcheck, fourvaried5000games(allframes), and30diagnostics
rerunning. Native defaulttraceOFF. CurrentPPU/BUSsourceisacceptednewrule.

Current38unit+ABI andwasmstreamhash checkscompleted: native/wasmFB
-1199743547,audio -194314846,87304samples for120calls. All20000images
ofvaried5000replays matchagain; samples3668206each, RMS/peaks unchanged
RFK .000738/1,RHDE .073714/2,FF .118294/1,Stallar .201956/3. Detailed
.out/dmc-selector-games.json. All30diagnosticimages/countsalsoexactagain,
maxaudioRMS .030205peak2 (.out/selector-dmc-diagnostic-comparison.json).

## Implicit one-byte DMC aborted reload fixed, later-chip extra absent

Newtools/dmc_implicit_stop_probe.py loadsonebyte nearbufferempty809, sample
C800FF/C80100 to distinguish repeated-address late-chipDMA. Enable796..807:
only800/801 differed (RMS~3.6); allothercasesincluding802/803alreadymatch.
Reference DOES NOT implement the late-chip unexpected full extra DMA for
implicitstop at808. DoNOTaddthis fromdocs; explicitstop808/809retainsfull
request aspreviouslyproved, butimplicitending808doesnot.

APU.supply now schedulesone-clock abortifthenonlooping lastbyte finishes
withbits1 andtimer1/2. Loopingsamplesdonotqualify. All12originalimplicit
probesnow RMS<.071peak1; two marker-gap6casesputCPUwriteontheabortedhalt
clock andalsomatch, provingcancellationonwrite. NewCPUunit actualmarker824
forenable800 and825forenable802protectsbothabortandnolater-chipfullDMA.
39units+ABI/wasmbuildchecks running;30diagnostics rerunningafterthischange.
NativecurrentdefaulttraceOFF. Sourcehasonlyvalidatedchanges, nofixtures
select-ROMdatabehavior. Nextwork: widerDMCreaderstart/stop/restartand
IRQ/OAMoverlap timing, OAMevaluationedges, exactgenericFIR/phase/mixer.

## OAMDATA bus during sprite fetch fixed; edge refinements validating

Originaltools/oam_eval_write_probe.py (28cases) provesactive2003writes
duringevaluation doNOTredirectthe evaluator inreference; nativealready
matches,so noeval_n/m synchronization change. OAMDATAreadatfetch does
differ: secondaryY/tile/attribute/X drive257/258/259/260 thenXheld261..264.
Slot1starts265 etc. CPUreadssee thelastprocessed dot. Previouslynative
alwaysreturnedprimaryOAM0Y duringfetch. PPU now updates eval_latch from
secondary onfetch andusesit throughcurrentdot321. All24initialrendered
readprobesnowexact (.out/oam-render-read/fixed-results.json).

Moreedgeprobes .out/oam-render-edge: currentdot1mustreturnprimaryOAM0Y60
(lastdot0), notFF; currentdot0stilllastsecondarybusFF. SecondaryclearFF
startsafterdot1 (currentdot2). Also primaryattrbits2..4readzero during
evaluation: attr1C->0, FF->E3. Themaskedbyteiscopiedintosecondary,sofetch
attralsomasked. ClearedemptysecondaryattrstillFF (dot284). PPU now uses
oam_read helpermaskingprimaryattr, readsprimaryatdot0, andstartsFFread
shortcutatcurrentdot2. These edge refinementsarebeingcompiled/validated.
Backup .out/ppu-before-oam-fetch-bus.rs. RequiredwasmstillpreOAMfetchfix
untilnewvalidation/testsarecomplete.

Audio: newuniform256MIDphase DAC16/64/96/127originaldatasets .out/filter-mid
look formodel errorsbelow1PCM. Relativegainratiosmatchf64mixerwithin~1ppm,
noevidenceforcoarselevelquantization. A naive second-moment estimatebiased
byADCquantizationtail; doNOTchangeglobalmixgainbasedonit. Nextfit direct
perphase FIR tables withmultipleamplitudes/PCMconstraints, heldoutdata.
CurrentvalidatedDSP unchanged; SRCgenericcubic spline .2RMS games.
