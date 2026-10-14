"""Record read-probe bytes in RAM, then display them as eight-pixel bars.

Changing the first DAC store to a RAM store preserves instruction timing.
The second DAC store remains. CNROM bank zero preserves the original CHR;
bank one is selected only after all reads, to display the captured bytes.
"""
import argparse,pathlib
p=argparse.ArgumentParser(description=__doc__);p.add_argument('source');p.add_argument('output');a=p.parse_args();src=pathlib.Path(a.source);r=bytearray(src.read_bytes());reads=[]
for i in range(16,16+32768-10):
 if r[i]==0xad and r[i+3:i+10]==bytes([0x8d,0x11,0x40,0x4a,0x8d,0x11,0x40]):
  n=len(reads);reads.append(dict(offset=i,register=f'{r[i+2]:02x}{r[i+1]:02x}'));r[i+4]=n&255;r[i+5]=2+(n>>8)
assert len(reads)<=512,len(reads)
end=int.from_bytes(r[16+32762:16+32764],'little');at=end-0x8000+16;code=bytearray()
def w(addr,v):code.extend([0xa9,v,0x8d,addr&255,addr>>8])
w(0x2001,0);code.extend([0x2c,2,0x20]);w(0x2000,0);assert r[17]&1;w(0x8001,1)
w(0x2006,0x3f);w(0x2006,0);w(0x2007,0);w(0x2007,21);w(0x2006,0x23);w(0x2006,0xc0);code.extend([0xa9,0,0xa2,64]);loop=len(code);code.extend([0x8d,7,0x20,0xca,0xd0]);code.append((loop-(len(code)+1))&255)
w(0x2006,0x20);w(0x2006,0)
for page in [2,3]:
 code.extend([0xa2,0]);loop=len(code);code.extend([0xbd,0,page,0x8d,7,0x20,0xe8,0xd0]);code.append((loop-(len(code)+1))&255)
w(0x2006,0);w(0x2006,0);w(0x2005,0);w(0x2005,0);w(0x2001,10);pc=end+len(code);code.extend([0x4c,pc&255,pc>>8]);assert at+len(code)<16+32762;r[at:at+len(code)]=code
r[5]=2;r[6]|=0x30;chr=bytearray(8192)
for tile in range(256):chr[tile*16:tile*16+8]=bytes([tile])*8
r.extend(chr);out=pathlib.Path(a.output);out.write_bytes(r);out.with_suffix('.reads.json').write_text(__import__('json').dumps(reads,indent=2))
print('captured',len(reads),'bytes')
