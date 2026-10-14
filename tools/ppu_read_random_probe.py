"""Original mixed PPU/controller reads and writes with DAC byte capture."""
import argparse,pathlib,subprocess,random,math
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--seed',type=int,default=0);p.add_argument('--events',type=int,default=800);a=p.parse_args();q=pathlib.Path(a.prefix);q.parent.mkdir(parents=True,exist_ok=True)
subprocess.run(['python3','tools/ppu_random_probe.py',str(q),'--seed',str(a.seed),'--events','0'],stdout=subprocess.DEVNULL,check=True);chr=q.with_suffix('.nes').read_bytes()[32784:];lines=q.with_suffix('.txt').read_text().splitlines();r=random.Random(a.seed^0x2c02);clock=70000
for i in range(a.events):
 clock+=r.choice([20,21,27,31,63,127,511,r.randrange(1000,7000)])
 if r.randrange(2):
  reg=r.choice([0x2000,0x2002,0x2004,0x2005,0x2006,0x2007,0x4015,0x4016,0x4017]);lines.append(f'READ {clock} {reg:04x}')
 else:
  reg=r.choice([0x2000,0x2001,0x2003,0x2004,0x2005,0x2006,0x2007,0x4016]);v=r.randrange(256)
  if reg==0x2000:v&=63
  if reg==0x2001:v=r.choice([0,8,16,30,31,62,94,158,254])
  lines.append(f'WRITE {clock} {reg:04x} {v}')
q.with_suffix('.txt').write_text('\n'.join(lines)+'\n');subprocess.run(['python3','tools/replay_noise.py',str(q.with_suffix('.txt')),str(q.with_suffix('.nes')),'--registers','2000,2001,2002,2003,2004,2005,2006,2007,4015,4016,4017'],check=True)
r=bytearray(q.with_suffix('.nes').read_bytes());r[5]=1;r[6]|=a.seed&1;r.extend(chr);q.with_suffix('.nes').write_bytes(r);q.with_suffix('.frames').write_text(str(math.ceil(clock/29780.5)+3))
