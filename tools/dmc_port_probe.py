"""Build an original absolute-port read immediately after timed DMC enable.

Primarily probes PPSTATUS reads near vblank, where an extra DMA read can
clear the flag before the real CPU access. The complete returned byte is
encoded in palette and emphasis, and its low7bits are written to the DAC.
"""
import argparse,pathlib,subprocess,sys
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('rom');p.add_argument('--enable-cycle',type=int,default=57169)
p.add_argument('--port',type=lambda v:int(v,0),default=0x2002)
p.add_argument('--nops',type=int,default=0)
a=p.parse_args();out=pathlib.Path(a.rom);out.parent.mkdir(parents=True,exist_ok=True)
rows=[(20,0x4017,64),(30000,0x4010,79),(30006,0x4011,127),
 (30012,0x4012,32),(30018,0x4013,1),(a.enable_cycle,0x4015,16)]
trace=out.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {addr:04x} {v}\n' for t,addr,v in rows))
subprocess.run([sys.executable,'tools/replay_noise.py',str(trace),str(out),
 '--registers','4010,4011,4012,4013,4015,4017'],check=True)
rom=bytearray(out.read_bytes());end=int.from_bytes(rom[-6:-4],'little');pos=end-0x8000
code=bytes([0xea])*a.nops+bytes([0xad,a.port&255,a.port>>8,0x85,0,
 0xa9,0,0x8d,0x15,0x40,0xa5,0,0x8d,0x11,0x40,0x29,0xc0,0x4a,0x8d,1,0x20,
 0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20,0xa5,0,0x29,0x3f,0x8d,7,0x20,
 0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20])
loop=end+len(code);code+=bytes([0x4c,loop&255,loop>>8]);rom[16+pos:16+pos+len(code)]=code
out.write_bytes(rom)
