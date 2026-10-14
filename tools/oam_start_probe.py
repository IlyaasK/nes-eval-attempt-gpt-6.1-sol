"""Original OAM address/enable probe with full-byte read capture support."""
import argparse,pathlib,subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--visual',action='store_true');p.add_argument('--visual-tile',type=int,default=162);p.add_argument('--eligible-next',action='store_true');p.add_argument('--all-eligible',action='store_true');p.add_argument('--eight-eligible',action='store_true');p.add_argument('--patterned',action='store_true');p.add_argument('--repeat-unaligned',action='store_true');p.add_argument('--first-read',type=int);p.add_argument('--address',type=int,default=1);p.add_argument('--write-cycle',type=int,default=77768);p.add_argument('--enable-cycle',type=int,default=40008);a=p.parse_args();q=pathlib.Path(a.prefix);q.parent.mkdir(parents=True,exist_ok=True)
rows=[(33000,0x2000,0),(33008,0x2001,0),(33016,0x2003,0)];oam=[255]*256;oam[:8]=[200,160,170,16,161,200,200,160]
if a.eligible_next:oam[4]=160
if a.patterned:
 for n in range(2,64):oam[n*4:n*4+4]=[200+n%32,210+n%32,224+n%32,240+n%16]
if a.repeat_unaligned:oam[4:8]=[161,160,170,18]
if a.all_eligible or a.eight_eligible:
 for n in range(64):oam[n*4:n*4+4]=[160 if a.all_eligible or n<8 else 200+n%32,64+n,128+n,192+n]
for i,v in enumerate(oam):rows.append((33024+8*i,0x2004,v))
if a.visual:
 rows.extend([(36000,0x2006,63),(36008,0x2006,0)])
 for i in range(32):rows.append((36016+8*i,0x2007,21 if i==17 else 0))
 rows.extend([(36400,0x2006,0),(36408,0x2006,0),(36416,0x2005,0),(36424,0x2005,0)])
rows.extend([(40000,0x2000,0),(40008,0x2001,30 if a.visual else 8)])
if a.enable_cycle!=40008:rows.extend([(77730,0x2001,0),(a.enable_cycle,0x2001,30 if a.visual else 8)])
rows.append((a.write_cycle,0x2003,a.address));first=a.first_read if a.first_read is not None else max(77770,a.write_cycle+4,a.enable_cycle+4)
# A one-cycle gap cannot be represented by the timing replay's delay code.
last=max(a.write_cycle,a.enable_cycle)
if first-last==5:first+=1
for i in range(4):rows.append((first+20*i,0x2004,None))
rows.sort();q.with_suffix('.txt').write_text(''.join(f'READ {t} {r:04x}\n' if v is None else f'WRITE {t} {r:04x} {v}\n' for t,r,v in rows));subprocess.run(['python3','tools/replay_noise.py',str(q.with_suffix('.txt')),str(q.with_suffix('.nes')),'--registers','2000,2001,2003,2004,2005,2006,2007'],check=True);r=bytearray(q.with_suffix('.nes').read_bytes());r[5]=1;chr=bytearray(8192)
if a.visual:chr[a.visual_tile*16:a.visual_tile*16+8]=bytes([255])*8
r.extend(chr);q.with_suffix('.nes').write_bytes(r)
