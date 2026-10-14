"""Build an original diagnostic for OAMADDR retained by forced blanking.

Rendering stops at scanline 61, dot 70. Evaluation has copied the first
sprite's Y and tile bytes and is pointing at its attribute byte. A $2004
write while blanked must overwrite byte 2. The final $2004 read encodes
that observation as palette $2f (correct) or $0f.
"""
import pathlib,subprocess,sys
out=pathlib.Path(sys.argv[1]);out.parent.mkdir(parents=True,exist_ok=True)
rows=[(20,0x2003,0)]
rows.extend((26+4*i,0x2004,255) for i in range(256))
rows.append((1060,0x2003,0))
rows.extend((1070+8*i,0x2004,v) for i,v in enumerate([60,1,0,80,60,1,1,100]))
rows.extend([(50000,0x2001,30),(66518,0x2001,0),(66524,0x2004,42),(66530,0x2003,2)])
trace=out.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {a:04x} {v}\n' for t,a,v in rows))
subprocess.run([sys.executable,'tools/replay_noise.py',str(trace),str(out),'--registers','2001,2003,2004'],check=True)
rom=bytearray(out.read_bytes());prg=rom[16:];end=int.from_bytes(prg[-6:-4],'little');p=end-0x8000
code=bytes([0xad,4,0x20,0x29,0x20,0x09,0x0f,0xaa,
 0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20,0x8a,0x8d,7,0x20,
 0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20])
loop=end+len(code);code+=bytes([0x4c,loop&255,loop>>8]);rom[16+p:16+p+len(code)]=code
out.write_bytes(rom)
