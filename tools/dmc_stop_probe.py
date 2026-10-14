"""Original DMC stop around the first buffer-empty clock809.

An audible DAC marker20CPUcycles after the intended stop exposes aborted
DMA's extra clock. A17byte sample also exposes a pending reload which is
retained when $4015 is cleared on the scheduling clock itself.
"""
import argparse,pathlib,subprocess,sys
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom')
p.add_argument('--stop-cycle',type=int,default=807)
p.add_argument('--length',type=int,default=1);a=p.parse_args()
out=pathlib.Path(a.rom);out.parent.mkdir(parents=True,exist_ok=True)
rows=[(16,0x4017,64),(22,0x4010,15),(28,0x4011,64),
 (34,0x4012,32),(40,0x4013,a.length),(46,0x4015,16),
 (a.stop_cycle-3,0x4015,0),(a.stop_cycle+20-3,0x4011,0)]
trace=out.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {addr:04x} {v}\n' for t,addr,v in rows))
subprocess.run([sys.executable,'tools/replay_noise.py',str(trace),str(out),
 '--registers','4010,4011,4012,4013,4015,4017'],check=True)
rom=bytearray(out.read_bytes());rom[16+0x4800:16+0x4900]=bytes([255])*256
out.write_bytes(rom)
