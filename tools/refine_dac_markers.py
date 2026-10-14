"""Infer individual oracle DAC-marker times, then verify with a CPU replay.

Use an original marker ROM which leaves triangle at its power DAC and uses
only $4011 for sound. This fits individual CPU cycles with the generic FIR;
verify inferred timestamps by an oracle DAC-only replay, not by fit error.
"""
import argparse,pathlib,re,wave,numpy as np
from scipy.signal import lfilter
from scipy.interpolate import BSpline
p=argparse.ArgumentParser(description=__doc__);p.add_argument('trace');p.add_argument('wav');p.add_argument('output');p.add_argument('--radius',type=int,default=8);a=p.parse_args()
with wave.open(a.wav) as w:pcm=np.frombuffer(w.readframes(w.getnframes()),'<i2').astype(float)
r=44100/1789773;r1=np.exp(-2*np.pi*90/44100);r2=np.exp(-2*np.pi*440/44100);alpha=1-np.exp(-2*np.pi*14000/44100)
u=lfilter([1,alpha-1],[alpha],pcm+np.sign(pcm)*.5);u=lfilter([1,-r2],[r2,-r2],u);u=lfilter([1,-r1],[r1,-r1],u);u=np.diff(u,prepend=0)
s=pathlib.Path('src/audio_kernel.rs').read_text();s=s[s.index('= [')+3:s.index('];')];coeff=np.array([float(x) for x in re.findall(r'[-+]?\d+\.\d+e[-+]\d+',s)]);knots=np.r_[[-18]*3,np.arange(-18,18+1/512,1/256),[18]*3];spline=BSpline(knots,coeff,3)
rows=[];previous=0;original=pathlib.Path(a.trace).read_text().splitlines()
def mix(v):return 32767*163.67/(24329/(45+v)+100)
for line in original:
 w=line.split()
 if w[:1]==['WRITE'] and w[2]=='4011':
  t,v=int(w[1]),int(w[3])&127;delta=mix(v)-mix(previous);previous=v
  if delta and 32<(t-8)*r<len(pcm)-50:rows.append((t,v,delta))
choices=np.arange(-a.radius,a.radius+1);models=[];slices=[];selection=np.full(len(rows),a.radius,dtype=int);pred=np.zeros(len(u))
for i,(t,v,delta) in enumerate(rows):
 ts=(t+choices-8)*r;quant=np.floor((ts+8*r)*256)/256-(ts+8*r)+1/512
 ids=np.floor((t-8)*r).astype(int)+np.arange(-19,21);x=ids[None,:]-ts[:,None]-quant[:,None]+1.30936
 kernels=spline(np.clip(x,-18,18));kernels[np.abs(x)>=16]=0;kernels/=kernels.sum(axis=1)[:,None];kernels*=delta
 models.append(kernels);slices.append(ids);pred[ids]+=kernels[a.radius]
residual=u-pred
for iteration in range(12):
 changes=0
 for i,kernels in enumerate(models):
  ids=slices[i];old=selection[i];without=residual[ids]+kernels[old];scores=np.sum((kernels-without[None,:])**2,axis=1);new=int(scores.argmin())
  if new!=old:residual[ids]+=kernels[old]-kernels[new];selection[i]=new;changes+=1
 print('iteration',iteration,'changed',changes,flush=True)
 if not changes:break
shifted={t:t+int(choices[j]) for (t,v,d),j in zip(rows,selection)};output=[]
for line in original:
 w=line.split()
 if w[:1]==['WRITE'] and w[2]=='4011' and int(w[1]) in shifted:w[1]=str(shifted[int(w[1])]);line=' '.join(w)
 output.append(line)
pathlib.Path(a.output).write_text('\n'.join(output)+'\n')
print('offset counts',np.unique(choices[selection],return_counts=True),'events',len(rows),flush=True)
