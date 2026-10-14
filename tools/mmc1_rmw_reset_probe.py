"""Original MMC1 consecutive data/reset writes produced by real CPU RMW."""
import argparse,pathlib
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom');p.add_argument('--value',type=lambda x:int(x,0),default=0x7f);p.add_argument('--opcode',type=lambda x:int(x,0),default=0xee);p.add_argument('--prefix',type=int,default=0);p.add_argument('--flags',type=lambda x:int(x,0),default=0x24);a=p.parse_args()
code=bytearray([0x78,0xa9,a.flags,0x48,0x28])
def serial(v,count=5):
 for bit in range(count):code.extend([0xa9,(v>>bit)&1,0x8d,0,0xe0])
serial(0,a.prefix)
code.extend([a.opcode,0,0xcf])
serial(2)
code.extend([0xad,0,0xb0,0x85,0])
# Wait beyond warm-up, then encode the byte without enabling rendering.
code.extend([0xa0,24,0xa2,255,0xca,0xd0,0xfd,0x88,0xd0,0xf8])
code.extend([0xa5,0,0x29,0xc0,0x4a,0x8d,1,0x20,0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20,0xa5,0,0x29,0x3f,0x8d,7,0x20,0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20])
end=0xc000+len(code);code.extend([0x4c,end&255,end>>8]);assert len(code)<0xf00
prg=bytearray().join(bytearray([17*(n+1)])*16384 for n in range(4));prg[-16384:-16384+len(code)]=code;prg[-16384+0xf00]=a.value
prg[-6:]=bytes([end&255,end>>8,0,0xc0,end&255,end>>8])
path=pathlib.Path(a.rom);path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'NES\x1a'+bytes([4,0,0x10,0])+bytes(8)+prg)
