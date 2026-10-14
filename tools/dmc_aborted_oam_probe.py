"""Probe a stopped DMC reload's retained read during a later sprite DMA.

An emphasis marker records the final CPU timing. Restarting DMC between
the abort and OAM tests whether a normal sample read consumes the state.
"""
import argparse,pathlib,subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--rate',type=int,default=15);p.add_argument('--loop',action='store_true');p.add_argument('--restart',action='store_true');p.add_argument('--ordinary-stop',action='store_true');p.add_argument('--oam-cycle',type=int);a=p.parse_args()
q=pathlib.Path(a.prefix);q.parent.mkdir(parents=True,exist_ok=True);period=[428,380,340,320,286,254,226,214,190,160,142,128,106,84,72,54][a.rate];byte=431+7*period;stop=byte-(100 if a.ordinary_stop else 3)
rows=[(16,0x4017,64),(22,0x4010,a.rate+64*a.loop),(28,0x4011,64),(34,0x4012,32),(40,0x4013,1),(46,0x4015,16),(stop-3,0x4015,0)]
if a.restart:rows.extend([(byte+1000,0x4015,16),(byte+1500,0x4015,0)])
rows.extend([(a.oam_cycle or byte+5000,0x4014,2),(33000,0x2001,58)])
q.with_suffix('.txt').write_text(''.join(f'WRITE {t} {r:04x} {v}\n' for t,r,v in rows));subprocess.run(['python3','tools/replay_noise.py',str(q.with_suffix('.txt')),str(q.with_suffix('.nes')),'--registers','4017,4010,4011,4012,4013,4015,4014,2001'],check=True)
r=bytearray(q.with_suffix('.nes').read_bytes());r[16+0x4800:16+0x4900]=bytes(256);q.with_suffix('.nes').write_bytes(r)
