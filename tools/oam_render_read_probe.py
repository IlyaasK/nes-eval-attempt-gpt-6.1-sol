"""Read OAMDATA at a selected rendering dot, then display the entire byte.

Reuse the original three-sprite fixture, with an OAMADDR0 write fourclocks
before the read. Sprite evaluation ignores these writes in the reference;
fetch/reset/clear phases also keep the address zero.
"""
import argparse,pathlib,subprocess,sys
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom')
p.add_argument('--read-cycle',type=int,default=66582)
p.add_argument('--attribute',type=int,default=0)
p.add_argument('--y',type=int,default=60)
p.add_argument('--tile',type=int,default=1)
p.add_argument('--control',type=int,default=0);a=p.parse_args()
out=pathlib.Path(a.rom)
subprocess.run([sys.executable,'tools/oam_eval_write_probe.py',str(out),
 '--write-cycle',str(a.read_cycle-4),'--address','0','--attribute',str(a.attribute),
 '--y',str(a.y),'--tile',str(a.tile),'--control',str(a.control)],check=True)
rom=bytearray(out.read_bytes());end=int.from_bytes(rom[-6:-4],'little');pos=end-0x8000
code=bytes([0xad,4,0x20,0x85,0,0xa9,0,0x8d,1,0x20,
 0xa5,0,0x29,0xc0,0x4a,0x8d,1,0x20,
 0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20,0xa5,0,0x29,0x3f,0x8d,7,0x20,
 0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20])
loop=end+len(code);code+=bytes([0x4c,loop&255,loop>>8]);rom[16+pos:16+pos+len(code)]=code
out.write_bytes(rom)
