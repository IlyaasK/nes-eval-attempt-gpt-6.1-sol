"""Build an original NROM diagnostic that replays timestamped noise writes."""
import sys,pathlib
rows=[]
for l in pathlib.Path(sys.argv[1]).read_text().splitlines():
 if not l.startswith('WRITE'):continue
 _,cycle,addr,value=l.split();cycle=int(cycle);addr=int(addr,16);value=int(value)
 if addr in [0x400c,0x400e,0x400f,0x4015,0x4017] and cycle<7_000_000:rows.append((cycle,addr,value))
code=bytearray([0x78]);clock=10;a=None

def emit(data,cycles):
 global clock
 code.extend(data);clock+=cycles

def delay(n):
 while n>3:
  if n>=6 and (len(code)&255) in [251,252,253]:emit([0x24,0],3);n-=3;continue
  count=min(255,(n-1)//5)
  if count==0:break
  cost=5*count+1
  if n-cost==1:count-=1;cost=5*count+1
  if count==0:break
  emit([0xa2,count,0xca,0xd0,0xfd],cost);n-=cost
 while n>=2:
  if n&1:emit([0x24,0],3);n-=3
  else:emit([0xea],2);n-=2
 assert n==0,n

initialized=False
for target,addr,value in rows:
 if target>10000 and not initialized:
  for v in range(256):emit([0xa9,v,0x85,v],5)
  a=255;initialized=True
 load=[] if a==value else [0xa9,value]
 required=4+(2 if load else 0);left=target-clock-required
 if left==1:
  assert initialized
  load=[0xa5,value];required=7;left=target-clock-required
 assert left>=0,(target,clock)
 delay(left)
 if load:emit(load,3 if load[0]==0xa5 else 2);a=value
 emit([0x8d,addr&255,addr>>8],4)
 assert clock==target
assert len(code)<32762,len(code)
end=0x8000+len(code);emit([0x4c,end&255,end>>8],3)
prg=code+bytes(32762-len(code))+bytes([end&255,end>>8,0,0x80,end&255,end>>8])
rom=b'NES\x1a'+bytes([2,0,0,0])+bytes(8)+prg
pathlib.Path(sys.argv[2]).write_bytes(rom)
print('bytes',len(code),'cycles',clock)
