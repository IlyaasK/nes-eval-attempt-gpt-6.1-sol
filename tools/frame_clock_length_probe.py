"""Observe whether a closely overlapping five-step reset clocks length."""
import argparse,pathlib,subprocess
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--write-cycle',type=int,default=7476);p.add_argument('--read-cycle',type=int,default=22397);a=p.parse_args();prefix=pathlib.Path(a.prefix);prefix.parent.mkdir(parents=True,exist_ok=True)
rows=[(18,0x4017,64),(30,0x4015,1),(50,0x4000,31),(60,0x4002,127),(70,0x4003,24),(a.write_cycle,0x4017,192),(a.read_cycle-4,0x2003,0)]
trace=prefix.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {reg:04x} {v}\n' for t,reg,v in rows))
rom=prefix.with_suffix('.nes');subprocess.run(['python3','tools/replay_noise.py',str(trace),str(rom),'--registers','4000,4002,4003,4015,4017,2003'],check=True)
b=bytearray(rom.read_bytes());end=int.from_bytes(b[-6:-4],'little');pos=16+end-0x8000
code=bytearray([0xad,0x15,0x40,0x29,1,0x85,0,0xa0,24,0xa2,255,0xca,0xd0,0xfd,0x88,0xd0,0xf8,0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20,0xa5,0,0x8d,7,0x20,0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20]);loop=end+len(code);code.extend([0x4c,loop&255,loop>>8]);b[pos:pos+len(code)]=code;rom.write_bytes(b)
