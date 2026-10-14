"""Original timed OAMADDR write near sprite fetch; read retained OAM byte.

OAM Y0=$11,Y2=$22; every other byte=$FF. The sampled byte is encoded
directly into the final forced-blank palette to expose row-copy corruption.
"""
import argparse,pathlib,subprocess,sys
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('rom');p.add_argument('--write-cycle',type=int,default=66580)
p.add_argument('--address',type=int,default=8)
p.add_argument('--read',type=int,default=0)
a=p.parse_args();out=pathlib.Path(a.rom);out.parent.mkdir(parents=True,exist_ok=True)
rows=[(20,0x2003,0)]
rows.extend((26+4*i,0x2004,255) for i in range(256))
rows.extend([(1060,0x2003,0),(1070,0x2004,17),(1080,0x2003,8),
 (1086,0x2004,34),(1092,0x2003,0),(50000,0x2001,8),
 (a.write_cycle,0x2003,a.address),(66610,0x2001,0),(66620,0x2003,a.read)])
trace=out.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {addr:04x} {v}\n' for t,addr,v in rows))
subprocess.run([sys.executable,'tools/replay_noise.py',str(trace),str(out),'--registers','2001,2003,2004'],check=True)
rom=bytearray(out.read_bytes());end=int.from_bytes(rom[-6:-4],'little');pos=end-0x8000
code=bytes([0xad,4,0x20,0xaa,0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20,
 0x8a,0x8d,7,0x20,0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20])
loop=end+len(code);code+=bytes([0x4c,loop&255,loop>>8]);rom[16+pos:16+pos+len(code)]=code
out.write_bytes(rom)
