"""Original initial DMC load across an output-byte boundary.

The first bit clock is431 at fixed power phase, so the first byte boundary
is431+7*period after a rate write before that bit clock. Multi-byte and
looping one-byte readers distinguish DMA cooldown from final-byte stops.
"""
import argparse,pathlib,subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--enable-cycle',type=int,default=804);p.add_argument('--length',type=int,default=1);p.add_argument('--rate',type=int,default=15);p.add_argument('--loop',action='store_true');p.add_argument('--marker-gap',type=int,default=0);p.add_argument('--first-byte',type=lambda x:int(x,0),default=0xa5);a=p.parse_args();prefix=pathlib.Path(a.prefix);prefix.parent.mkdir(parents=True,exist_ok=True)
rows=[(16,0x4017,64),(22,0x4010,a.rate+(64 if a.loop else 0)),(28,0x4011,64),(34,0x4012,32),(40,0x4013,a.length),(a.enable_cycle,0x4015,16)]
if a.marker_gap:rows.append((a.enable_cycle+a.marker_gap,0x4011,0))
trace=prefix.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {addr:04x} {v}\n' for t,addr,v in rows));rom=prefix.with_suffix('.nes');subprocess.run(['python3','tools/replay_noise.py',str(trace),str(rom),'--registers','4010,4011,4012,4013,4015,4017'],check=True)
b=bytearray(rom.read_bytes());b[16+0x4800:16+0x4804]=bytes([a.first_byte,0x5a,0x3c,0xc3]);rom.write_bytes(b)
