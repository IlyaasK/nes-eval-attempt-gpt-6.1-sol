"""Probe controller reads after an OAM DMA crossing the frame callback.

Hold A during call 2 and release it before call 3. The read immediately
after the transfer must complete with call 2 and publish palette colour 33.
"""
import pathlib,subprocess,numpy as np
root=pathlib.Path('.out/dma-frame-input');root.mkdir(exist_ok=True)
(root/'keys.txt').write_text('2 01\n3 00\n')
for t in [59100,59400,59500,59550]:
 p=root/f'oam{t}';p.with_suffix('.txt').write_text(f'WRITE 100 4016 1\nWRITE 33000 2001 32\nWRITE {t} 4014 2\n')
 subprocess.run(['python3','tools/replay_noise.py',str(p.with_suffix('.txt')),str(p.with_suffix('.nes')),'--registers','2001,4014,4016'],stdout=subprocess.DEVNULL,check=True)
 rom=bytearray(p.with_suffix('.nes').read_bytes());j=rom.find(bytes([0x8d,0x14,0x40]))+3
 code=bytes([0xad,0x16,0x40,0x29,1,0x85,0,0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20,0xa5,0,0x18,0x69,0x20,0x8d,7,0x20,0xa9,0,0x8d,6,0x20,0x8d,6,0x20]);end=j-16+0x8000+len(code);code+=bytes([0x4c,end&255,end>>8]);rom[j:j+len(code)]=code;p.with_suffix('.nes').write_bytes(rom)
 subprocess.run(['oracle','run',str(p.with_suffix('.nes')),'5','--replay',str(root/'keys.txt'),'--dump-final',str(p)+'-ref'],stdout=subprocess.DEVNULL,check=True)
 for v,b in [('native','target/release/examples/run')]:
  subprocess.run([b,str(p.with_suffix('.nes')),'5',str(p)+'-'+v,str(root/'keys.txt')],stdout=subprocess.DEVNULL,check=True)
  a=np.fromfile(str(p)+'-'+v+'/frame_00004.idx','<u2');r=np.fromfile(str(p)+'-ref/frame_00004.idx','<u2');print(t,v,int((a!=r).sum()),np.unique(a),np.unique(r),flush=True)
