# NES emulator

Rust NES emulator for the offline ABI in `spec/ABI.md`.

```
cargo build --release --lib --target wasm32-unknown-unknown
```

The output is `target/wasm32-unknown-unknown/release/nes_emu.wasm`. It exports
all ten ABI functions and linear memory, and requires no host imports.

The CPU implements all official and unofficial opcodes, with individual bus
accesses, DMA, and interrupt polling. CPU instructions are Rust futures that
suspend between bus cycles; the scheduler drives the PPU three dots and the
APU one cycle per CPU bus cycle. The PPU uses background shifters, secondary
OAM evaluation, sprite fetches, scrolling registers, and palette indices.
Cartridges support NROM, MMC1, UxROM, CNROM, and MMC3, including CHR RAM,
MMC1 outer PRG banking, MMC3 bank inversion and IRQs, and NES 2.0 RAM sizes.
Audio uses the five APU channels, nonlinear mixing, fractional-clock
band-limited resampling, and high/low-pass filters.

The implementation is under active development and is not fully accurate.
The bundled oracle is the compatibility target. Its first `run_frame` call
returns the initialized framebuffer without executing CPU cycles; this was
verified with a ROM that samples a button at its first instructions. The
frame wrapper completes its current CPU instruction, and the framebuffer
is updated in place, matching the oracle's observable top-row behavior.

## Development tools

```
cargo test
cargo run --release --example run -- dev-roms/robotfindskitten.nes 660 .out/run tools/gameplay.txt
oracle run dev-roms/robotfindskitten.nes 660 --replay tools/gameplay.txt --dump-frames .out/reference --dump-audio .out/reference.wav
python3 tools/compare.py .out/run .out/reference
python3 tools/wasm_check.py dev-roms/robotfindskitten.nes 660 tools/gameplay.txt
```

The native runner writes palette-index frames and raw little-endian i16 audio.
Set `LAST_ONLY=1` to save only the final image. The wasm checker uploads a ROM
into the exported buffer, runs input and frame calls in wasmtime, checks audio
draining, and prints framebuffer/audio FNV hashes, sample count, and rate.
It uses a separate WAT harness and does not add imports to the emulator.

The assembly probes in `tools/` are original diagnostic ROMs. They isolate
startup input, DAC transitions, and individual audio channels. Their linker
configuration is `tools/nrom.cfg`. The optional `trace` feature logs CPU/APU
writes and mixer transitions for native debugging.
