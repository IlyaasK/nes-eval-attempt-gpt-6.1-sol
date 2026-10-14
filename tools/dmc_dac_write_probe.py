"""Original DMC DAC write at/near the first active output-unit clock.

The single-byte nonlooping sample has only its initial three-cycle DMA.
Subtract those three cycles from the generated CPU delay to place the
write at the requested absolute clock (default863, the first DAC edge).
"""
import argparse,pathlib,subprocess,sys
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('rom');p.add_argument('--old',type=int,default=64)
p.add_argument('--value',type=int,default=64)
p.add_argument('--sample',type=int,default=255)
p.add_argument('--write-cycle',type=int,default=863)
a=p.parse_args();out=pathlib.Path(a.rom);out.parent.mkdir(parents=True,exist_ok=True)
rows=[(16,0x4017,64),(22,0x4010,15),(28,0x4011,a.old),
 (34,0x4012,32),(40,0x4013,0),(46,0x4015,16),
 (a.write_cycle-3,0x4011,a.value)]
trace=out.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {addr:04x} {v}\n' for t,addr,v in rows))
subprocess.run([sys.executable,'tools/replay_noise.py',str(trace),str(out),
 '--registers','4010,4011,4012,4013,4015,4017'],check=True)
rom=bytearray(out.read_bytes());rom[16+0x4800]=a.sample;out.write_bytes(rom)
