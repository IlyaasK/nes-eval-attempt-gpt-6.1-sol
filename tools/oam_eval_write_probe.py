"""Original mid-evaluation OAMADDR write with three visibly distinct sprites.

Solid CHR tiles; sprites0,1,32 atx40/80/160,Y60 anddifferentpalettes. The
CPU writes OAMADDR duringline61 evaluation; capture frame3 to see whether
it redirects the evaluator used forline62. Dot70 corresponds toCPU66518.
"""
import argparse,pathlib,subprocess,sys
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom')
p.add_argument('--write-cycle',type=int,default=66518)
p.add_argument('--address',type=int,default=128)
p.add_argument('--attribute',type=int,default=0);a=p.parse_args()
out=pathlib.Path(a.rom);out.parent.mkdir(parents=True,exist_ok=True)
rows=[(20,0x2003,0)]
rows.extend((26+4*i,0x2004,255) for i in range(256))
clock=1070
for addr,y,tile,attr,x in [(0,60,1,a.attribute,40),(4,60,2,1,80),(128,60,3,2,160)]:
 for reg,v in [(0x2003,addr),(0x2004,y),(0x2004,tile),(0x2004,attr),(0x2004,x)]:
  rows.append((clock,reg,v));clock+=6
rows.append((clock,0x2003,0));clock+=10
rows.extend((clock+4*i,0x2007,255) for i in range(128))
clock=33000
for addr,v in [(0x3f00,15),(0x3f13,17),(0x3f17,33),(0x3f1b,49)]:
 for reg,val in [(0x2006,addr>>8),(0x2006,addr&255),(0x2007,v)]:
  rows.append((clock,reg,val));clock+=6
rows.extend([(34000,0x2006,0),(34004,0x2006,0),(50000,0x2001,20),
 (a.write_cycle,0x2003,a.address)])
trace=out.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {addr:04x} {v}\n' for t,addr,v in rows))
subprocess.run([sys.executable,'tools/replay_noise.py',str(trace),str(out),
 '--registers','2001,2003,2004,2006,2007'],check=True)
