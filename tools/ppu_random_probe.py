"""Build original seeded PPU register streams with patterned CHR/nametables.

The CPU executes every scheduled write, including mid-render PPUADDR,
PPUDATA, OAM, scroll, mask and control changes. DAC markers share the bus.
"""
import argparse,pathlib,random,subprocess,math
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--seed',type=int,default=0);p.add_argument('--events',type=int,default=600);a=p.parse_args()
q=pathlib.Path(a.prefix);q.parent.mkdir(parents=True,exist_ok=True);r=random.Random(a.seed);rows=[];clock=33000
def write(reg,value):
 global clock
 rows.append((clock,reg,value));clock+=8
write(0x2000,0);write(0x2001,0);write(0x2006,0x3f);write(0x2006,0)
for i in range(32):write(0x2007,r.randrange(64))
write(0x2006,0x20);write(0x2006,0)
for i in range(2048):write(0x2007,r.randrange(256))
write(0x2003,0)
for i in range(256):write(0x2004,r.randrange(256))
write(0x2006,0);write(0x2006,0);write(0x2005,0);write(0x2005,0);write(0x2000,0);write(0x2001,30)
clock=max(clock,70000)
for i in range(a.events):
 clock+=r.choice([8,9,13,17,31,63,127,511,r.randrange(1000,7000)])
 reg=r.choice([0x2000,0x2001,0x2003,0x2004,0x2005,0x2006,0x2007,0x4011]);val=r.randrange(256)
 if reg==0x2001:val=r.choice([0,8,16,24,30,31,62,94,158,254])
 if reg==0x2000:val=r.randrange(64)
 if reg==0x4011:val&=127
 rows.append((clock,reg,val))
q.with_suffix('.txt').write_text(''.join(f'WRITE {t} {reg:04x} {v}\n' for t,reg,v in rows))
subprocess.run(['python3','tools/replay_noise.py',str(q.with_suffix('.txt')),str(q.with_suffix('.nes')),'--registers','2000,2001,2003,2004,2005,2006,2007,4011'],check=True)
rom=bytearray(q.with_suffix('.nes').read_bytes());rom[5]=1;rom[6]|=a.seed&1;rom.extend(bytes(r.randrange(256) for _ in range(8192)));q.with_suffix('.nes').write_bytes(rom);q.with_suffix('.frames').write_text(str(math.ceil(clock/29780.5)+3))
