"""Build original patterned PPU initialization and one timed register change.

Use --kind addr to probe the two-clock external fetch around a delayed
PPUADDR transfer, or --kind enable to probe the first background tile.
"""
import argparse,pathlib,subprocess,math
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--kind',choices=['addr','enable','data','read'],default='addr');p.add_argument('--cycle',type=int,default=84940);p.add_argument('--seed',type=int,default=0);p.add_argument('--address',type=lambda x:int(x,0),default=0x224f);p.add_argument('--prime-address',action='store_true');p.add_argument('--value',type=lambda x:int(x,0),default=128);p.add_argument('--chr-value',type=lambda x:int(x,0));a=p.parse_args()
q=pathlib.Path(a.prefix);q.parent.mkdir(parents=True,exist_ok=True)
subprocess.run(['python3','tools/ppu_random_probe.py',str(q),'--seed',str(a.seed),'--events','0'],check=True)
chr=q.with_suffix('.nes').read_bytes()[16+32768:];lines=q.with_suffix('.txt').read_text().splitlines()
if a.kind=='enable':
 lines=[l for l in lines if l!='WRITE 51784 2001 30'];lines.append(f'WRITE {a.cycle} 2001 10')
elif a.kind=='read':
 lines.extend([f'READ {a.cycle} 2007',f'WRITE {a.cycle+20} 2001 0',f'READ {a.cycle+40} 2007'])
elif a.kind=='data':
 if a.prime_address:lines.extend([f'WRITE {a.cycle-40} 2006 {a.address>>8}',f'WRITE {a.cycle-20} 2006 {a.address&255}'])
 lines.append(f'WRITE {a.cycle} 2007 {a.value}')
else:
 lines.extend([f'WRITE {a.cycle-20} 2006 {a.address>>8}',f'WRITE {a.cycle} 2006 {a.address&255}'])
q.with_suffix('.txt').write_text('\n'.join(lines)+'\n')
subprocess.run(['python3','tools/replay_noise.py',str(q.with_suffix('.txt')),str(q.with_suffix('.nes')),'--registers','2000,2001,2003,2004,2005,2006,2007'],check=True)
rom=bytearray(q.with_suffix('.nes').read_bytes());rom[5]=1;rom[6]|=a.seed&1;rom.extend(chr if a.chr_value is None else bytes([a.chr_value])*8192);q.with_suffix('.nes').write_bytes(rom);q.with_suffix('.frames').write_text(str(math.ceil(a.cycle/29780.5)+1))
