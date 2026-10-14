"""Original timed sprite-size/table change with one selected sprite."""
import argparse,pathlib,subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--cycle',type=int,default=77949);p.add_argument('--initial',type=lambda x:int(x,0),default=32);p.add_argument('--next',type=lambda x:int(x,0),default=0);p.add_argument('--y',type=int,default=152);p.add_argument('--tile',type=int,default=0);p.add_argument('--zero-addresses',default='');a=p.parse_args();q=pathlib.Path(a.prefix)
subprocess.run(['python3','tools/ppu_random_probe.py',str(q),'--events','0'],stdout=subprocess.DEVNULL,check=True);rows=[];pi=oi=0
for line in q.with_suffix('.txt').read_text().splitlines():
 _,clock,reg,value=line.split();clock=int(clock);reg=int(reg,16);value=int(value)
 if reg==0x2007 and 33032<=clock<33288:value=1 if pi<16 or pi%4==0 else pi;pi+=1
 if reg==0x2004:value=[a.y,a.tile,0,96][oi] if oi<4 else (255 if oi%4==0 else 0);oi+=1
 if clock==51776:value=a.initial
 rows.append((clock,reg,value))
rows.append((a.cycle,0x2000,a.next));q.with_suffix('.txt').write_text(''.join(f'WRITE {c} {r:04x} {v}\n' for c,r,v in rows));subprocess.run(['python3','tools/replay_noise.py',str(q.with_suffix('.txt')),str(q.with_suffix('.nes')),'--registers','2000,2001,2003,2004,2005,2006,2007'],stdout=subprocess.DEVNULL,check=True)
r=bytearray(q.with_suffix('.nes').read_bytes());r[5]=1;chr=bytearray([255])*8192
for addr in a.zero_addresses.split(','):
 if addr:chr[int(addr,0)]=0
r.extend(chr);q.with_suffix('.nes').write_bytes(r)
