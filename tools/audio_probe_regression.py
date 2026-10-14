"""Compare retained original APU/DMA streams, sample counts and final video."""
import argparse,pathlib,subprocess,os,json,numpy as np,wave,concurrent.futures
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',default='target/release/examples/run');p.add_argument('--metadata',default='.out/dmc-latch-random-regression.json');p.add_argument('--output',default='.out/audio-probe-regression.json');a=p.parse_args()
def run(item):
 rom=pathlib.Path(item['rom'])
 with wave.open(str(rom.with_suffix('.wav'))) as w:r=np.frombuffer(w.readframes(w.getnframes()),'<i2').astype(float)
 # Older DMC/OAM probes ran extra trailing frames beyond their generated
 # schedule. Recover the actual call count from the retained reference.
 frames=item.get('frames',round((len(r)+17)*1789773/(44100*29780.5))+1)
 dest=rom.parent/(rom.stem+'-audio-regression');subprocess.run([a.binary,str(rom),str(frames),str(dest)],env=dict(os.environ,LAST_ONLY='1'),stdout=subprocess.DEVNULL,check=True)
 n=np.fromfile(dest/'audio.raw','<i2').astype(float)
 row=dict(rom=str(rom),samples=len(n),ref_samples=len(r))
 if len(n)==len(r):e=n-r;row.update(rms=float(np.sqrt(np.mean(e*e))),peak=float(abs(e).max()))
 ref=rom.parent/(rom.stem+'-ref')/f'frame_{frames-1:05}.idx'
 if ref.exists():row['pixels']=int((np.fromfile(dest/ref.name,'<u2')!=np.fromfile(ref,'<u2')).sum())
 if row.get('peak',999)>3 or row.get('pixels',0):print(row,flush=True)
 return row
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:rows=list(ex.map(run,json.loads(pathlib.Path(a.metadata).read_text())))
pathlib.Path(a.output).write_text(json.dumps(rows,indent=2));print('total',len(rows),'badcounts',sum(x['samples']!=x['ref_samples'] for x in rows),'badimages',sum(x.get('pixels',0)!=0 for x in rows),'worst',max(rows,key=lambda x:x.get('rms',999)),flush=True)
