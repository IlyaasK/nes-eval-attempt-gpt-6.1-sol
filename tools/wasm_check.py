"""Exercise the exported ABI under wasmtime, without any WASI imports."""
import pathlib,sys,subprocess
rom=pathlib.Path(sys.argv[1]).read_bytes();frames=int(sys.argv[2])
changes=[]
if len(sys.argv)>3:
 for line in pathlib.Path(sys.argv[3]).read_text().splitlines():
  words=line.split('#')[0].split()
  if len(words)==2:changes.append((int(words[0]),int(words[1],16)))
inputs='\n'.join(f'(if (i32.eq (local.get $frame) (i32.const {f})) (then (call $keys (i32.const {k}))))' for f,k in changes)
blob=''.join('\\%02x'%b for b in rom)
wat=f'''(module
(import "nes" "memory" (memory 1))
(import "nes" "emu_init" (func $init (result i32)))
(import "nes" "emu_rom_buffer" (func $rom (result i32)))
(import "nes" "emu_load_rom" (func $load (param i32) (result i32)))
(import "nes" "emu_reset" (func $reset (result i32)))
(import "nes" "emu_set_keys" (func $keys (param i32)))
(import "nes" "emu_run_frame" (func $run))
(import "nes" "emu_framebuffer" (func $fb (result i32)))
(import "nes" "emu_audio_buffer" (func $audio (result i32)))
(import "nes" "emu_audio_samples" (func $samples (result i32)))
(import "nes" "emu_audio_rate" (func $rate (result i32)))
(data $romdata "{blob}")
(func $hash (param $ptr i32) (param $n i32) (param $value i32) (result i32)
 (local $i i32)
 (block $end (loop $again
  (br_if $end (i32.ge_u (local.get $i) (local.get $n)))
  (local.set $value (i32.mul (i32.xor (local.get $value) (i32.load16_u (i32.add (local.get $ptr) (i32.shl (local.get $i) (i32.const 1))))) (i32.const 16777619)))
  (local.set $i (i32.add (local.get $i) (i32.const 1))) (br $again)))
 (local.get $value))
(func (export "check") (result i32 i32 i32 i32)
 (local $frame i32) (local $count i32) (local $n i32) (local $ahash i32)
 (if (i32.ne (call $init) (i32.const 1)) (then unreachable))
 (memory.init $romdata (call $rom) (i32.const 0) (i32.const {len(rom)}))
 (if (i32.ne (call $load (i32.const {len(rom)})) (i32.const 1)) (then unreachable))
 (local.set $ahash (i32.const -2128831035))
 (loop $frames
  {inputs}
  (call $run)
  (local.set $n (call $samples))
  (local.set $count (i32.add (local.get $count) (local.get $n)))
  (local.set $ahash (call $hash (call $audio) (local.get $n) (local.get $ahash)))
  (if (i32.ne (call $samples) (i32.const 0)) (then unreachable))
  (local.set $frame (i32.add (local.get $frame) (i32.const 1)))
  (br_if $frames (i32.lt_u (local.get $frame) (i32.const {frames}))))
 (call $hash (call $fb) (i32.const 61440) (i32.const -2128831035))
 (local.get $ahash) (local.get $count) (call $rate))
)
'''
path=pathlib.Path('.out/abi-check.wat');path.parent.mkdir(exist_ok=True);path.write_text(wat)
subprocess.run(['wasmtime','run','--preload','nes=target/wasm32-unknown-unknown/release/nes_emu.wasm','--invoke','check',str(path)],check=True)
