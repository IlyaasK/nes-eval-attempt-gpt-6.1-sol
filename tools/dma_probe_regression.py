"""Re-run retained original DMA-side-effect probes against saved references."""
import argparse,pathlib,subprocess,os,json,numpy as np,wave
p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',default='target/release/examples/run');p.add_argument('--metadata',default='.out/dmc-selector-regression.json');p.add_argument('--output',default='.out/dmc-boundary-dma-regression.json');a=p.parse_args();results=[]
for item in json.loads(pathlib.Path(a.metadata).read_text()):
 rom=pathlib.Path(item['rom']);refdir=next((d for d in [rom.parent/(rom.stem+'-ref'),pathlib.Path(str(rom)+'-ref')] if list(d.glob('frame_*.idx'))),None);assert refdir,rom
 refs=sorted(refdir.glob('frame_*.idx'))
 frame=int(refs[-1].stem.split('_')[1]);ref=refs[-1].read_bytes();dest=rom.parent/(rom.stem+'-load-boundary');replay='tools/controller-dmc-input.txt'
 args=[a.binary,str(rom),str(frame+1),str(dest),replay]
 subprocess.run(args,env=dict(os.environ,LAST_ONLY='1'),stdout=subprocess.DEVNULL,check=True)
 native=(dest/refs[-1].name).read_bytes();pixels=int(np.count_nonzero(np.frombuffer(native,'<u2')!=np.frombuffer(ref,'<u2')))
 row=dict(rom=str(rom),pixels=pixels,frames=frame+1)
 wav=next((f for f in [rom.parent/(rom.stem+'-ref.wav'),rom.with_suffix('.wav')] if f.exists()),None)
 if wav:
  n=np.fromfile(dest/'audio.raw','<i2').astype(float)
  with wave.open(str(wav)) as w:r=np.frombuffer(w.readframes(w.getnframes()),'<i2').astype(float)
  if len(n)==len(r):
   d=n-r;row.update(rms=float(np.sqrt(np.mean(d*d))),peak=float(abs(d).max()))
  else:row.update(native_samples=len(n),ref_samples=len(r))
 results.append(row)
 if pixels or row.get('peak',0)>5:print(row,flush=True)
pathlib.Path(a.output).write_text(json.dumps(results,indent=2));print('total',len(results),'bad images',sum(r['pixels']!=0 for r in results),'max audio peak',max(r.get('peak',0) for r in results),flush=True)
