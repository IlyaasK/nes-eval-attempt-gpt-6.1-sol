"""Original envelope stream with a five-step reset near a scheduled clock."""
import argparse,pathlib,subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--write-cycle',type=int,default=7476);p.add_argument('--first-mode',type=int,default=64);p.add_argument('--channel',choices=['noise','pulse'],default='noise');a=p.parse_args();out=pathlib.Path(a.prefix);out.parent.mkdir(parents=True,exist_ok=True)
rows=[(18,0x4017,a.first_mode)]
if a.channel=='noise':rows +=[(30,0x4015,8),(50,0x400c,32),(60,0x400e,3),(70,0x400f,0)]
else:rows +=[(30,0x4015,1),(50,0x4000,32),(60,0x4002,127),(70,0x4003,0)]
rows.append((a.write_cycle,0x4017,192))
trace=out.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {reg:04x} {v}\n' for t,reg,v in rows));subprocess.run(['python3','tools/replay_noise.py',str(trace),str(out.with_suffix('.nes')),'--registers','4000,4002,4003,400c,400e,400f,4015,4017'],check=True)
