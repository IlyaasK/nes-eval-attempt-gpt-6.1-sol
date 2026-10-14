"""Original one-byte DMC loads which finish just before buffer transfer809.

OnlyC800=$FF; C801=$00 distinguishes a repeated last-address fetch from
an ordinary next-byte fetch. A DAC marker20program clocks after enable
exposes an aborted reload or a full extra DMA without IRQ polling.
"""
import argparse,pathlib,subprocess,sys
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom')
p.add_argument('--enable-cycle',type=int,default=800)
p.add_argument('--marker-gap',type=int,default=20);a=p.parse_args()
out=pathlib.Path(a.rom);out.parent.mkdir(parents=True,exist_ok=True)
rows=[(16,0x4017,64),(22,0x4010,15),(28,0x4011,64),
 (34,0x4012,32),(40,0x4013,0),(a.enable_cycle,0x4015,16),
 (a.enable_cycle+a.marker_gap,0x4011,0)]
trace=out.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {addr:04x} {v}\n' for t,addr,v in rows))
subprocess.run([sys.executable,'tools/replay_noise.py',str(trace),str(out),
 '--registers','4010,4011,4012,4013,4015,4017'],check=True)
rom=bytearray(out.read_bytes());rom[16+0x4800]=255;out.write_bytes(rom)
