"""Isolate the first dense-stream discrepancy at CPU108579.

Keep DMC setup/output-reader phase from ppu_dmc_stream_probe.s, replacing
earlier streaming with CPU delay. Reproduce its final PPUDATA reads and
DAC writes at108562/108566/108575/108579, then encode the next read in video.
The 723 prior DMA clocks comprise the initial3 plus180 reloads of4.
"""
import argparse,pathlib,subprocess,sys
p=argparse.ArgumentParser(description=__doc__);p.add_argument('rom')
p.add_argument('--chr-ram',action='store_true');p.add_argument('--shift',type=int,default=0)
p.add_argument('--length',type=int,default=1)
p.add_argument('--store',type=lambda v:int(v,0),default=0x4011)
p.add_argument('--store-y',type=int)
p.add_argument('--read-port',type=lambda v:int(v,0),default=0x2007)
p.add_argument('--after-port',type=lambda v:int(v,0),default=0x2007)
p.add_argument('--edge-cycle',type=int,default=108579)
p.add_argument('--intervening-read',action='store_true')
p.add_argument('--intervening-ppu',type=lambda v:int(v,0));a=p.parse_args()
out=pathlib.Path(a.rom);out.parent.mkdir(parents=True,exist_ok=True)
dma_clocks=3+4*((a.edge_cycle-1-30819)//432+1)
rows=[(16,0x4017,64),(30777,0x4010,79),(30783,0x4012,32),
 (30789,0x4013,a.length),(30795,0x4015,16),
 (a.edge_cycle-59-dma_clocks,0x2006,0),(a.edge_cycle-53-dma_clocks,0x2006,6)]
if a.chr_ram:
 rows.extend([(30000,0x2006,0),(30006,0x2006,6)])
 rows.extend((30012+i*6,0x2007,(6+i+a.shift)&127) for i in range(4))
 rows.sort()
trace=out.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {addr:04x} {v}\n' for t,addr,v in rows))
subprocess.run([sys.executable,'tools/replay_noise.py',str(trace),str(out),
 '--registers','2006,2007,4010,4012,4013,4015,4017'],check=True)
rom=bytearray(out.read_bytes());end=int.from_bytes(rom[-6:-4],'little');pos=end-0x8000
rom[16+0x4800:16+0x4900]=bytes([255])*256
port=[a.read_port&255,a.read_port>>8]
code=bytes([0xad,*port,0xa2,24])
wait=13-(a.intervening_ppu is not None)
code+=(bytes([0xa0,a.store_y])+bytes([0xea])*(wait-1)) if a.store_y is not None else bytes([0xea])*wait
code+=bytes([0xad,*port,0x8d,0x11,0x40])
code+=bytes([0x24,0]) if a.intervening_ppu is not None else bytes([0xea]) if a.intervening_read else bytes([0x24,0,0xea])
code+=bytes([0xad,*port])
if a.intervening_read:code+=bytes([0x24,0])
if a.intervening_ppu is not None:code+=bytes([0x2c,a.intervening_ppu&255,a.intervening_ppu>>8])
code+=bytes([0x8c if a.store_y is not None else 0x8d,a.store&255,a.store>>8,0xca,0xd0,0,
 0xad,a.after_port&255,a.after_port>>8,0x85,0,0xa9,0,0x8d,0x15,0x40,
 0xa5,0,0x29,0xc0,0x4a,0x8d,1,0x20,
 0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20,0xa5,0,0x29,0x3f,0x8d,7,0x20,
 0xa9,0x3f,0x8d,6,0x20,0xa9,0,0x8d,6,0x20])
loop=end+len(code);code+=bytes([0x4c,loop&255,loop>>8]);rom[16+pos:16+pos+len(code)]=code
if not a.chr_ram:
 rom[5]=1;rom.extend(bytes((i+a.shift)&127 for i in range(8192)))
out.write_bytes(rom)
