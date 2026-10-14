"""Fit PPU address-pin phases from an original MMC1 RAM/DAC probe.

RAM_READ trace rows record the CPU read time and next PPU dot. The probe
stores $11 and $22 in two RAM banks and immediately writes each read to
$4011 (four CPU cycles later). Compare generic pin-timing models against the
reference WAV using the calibrated reconstruction filter, then verify the
best model in the emulator. This does not use a reference source or state.
"""
import argparse,pathlib,re,wave,itertools,numpy as np
from scipy.signal import lfilter
from scipy.interpolate import BSpline
p=argparse.ArgumentParser(description=__doc__);p.add_argument('trace');p.add_argument('wav');p.add_argument('--ctrl',type=int,default=16);a=p.parse_args()
rows=[list(map(int,l.split()[1:])) for l in pathlib.Path(a.trace).read_text().splitlines() if l.startswith('RAM_READ ')]
rows=np.array(rows);t=rows[:,0]+4;scan=rows[:,2];dot=rows[:,3]-1;scan=np.where(dot<0,(scan-1)%262,scan);dot=dot%341
with wave.open(a.wav) as w:pcm=np.frombuffer(w.readframes(w.getnframes()),'<i2').astype(float)
r=44100/1789773;ts=(t-8)*r;valid=ts<len(pcm)-20;rows=rows[valid];t=t[valid];ts=ts[valid];scan=scan[valid];dot=dot[valid]
s=pathlib.Path('src/audio_kernel.rs').read_text();s=s[s.index('= [')+3:s.index('];')];coeff=np.array([float(x) for x in re.findall(r'[-+]?\d+\.\d+e[-+]\d+',s)]);knots=np.r_[[-18]*3,np.arange(-18,18+1/512,1/256),[18]*3];f=BSpline(knots,coeff,3)
q=np.floor((ts+8*r)*256)/256-(ts+8*r)+1/512;ids=np.floor(ts).astype(int)[:,None]+np.arange(-19,20)[None,:];x=ids-ts[:,None]-q[:,None]+1.30936
k=f(np.clip(x,-18,18));k[np.abs(x)>=16]=0;k/=k.sum(axis=1)[:,None];flat=ids.ravel();n=len(pcm)
r1=np.exp(-2*np.pi*90/44100);r2=np.exp(-2*np.pi*440/44100);alpha=1-np.exp(-2*np.pi*14000/44100);den=np.convolve(np.convolve([1,-r1],[1,-r2]),[1,alpha-1]);num=np.array([1,-1])*r1*r2*alpha
startup=pathlib.Path('src/audio.rs').read_text();startup=startup[startup.index('const STARTUP'):];startup=startup[startup.index('= [')+3:startup.index('];')];cdf=np.array([float(v) for v in re.findall(r'[-+]?\d+\.\d+(?:e[-+]?\d+)?',startup)]);bootcdf=np.r_[cdf[:16],np.ones(n-16)];boot=lfilter(np.array([1,-2,1])*r1*r2*alpha,den,bootcdf)*(32767*163.67/(24329/45+100))
def sound(value):
 levels=32767*163.67/(24329/(45+value)+100);delta=np.diff(levels,prepend=32767*163.67/(24329/45+100))
 stream=np.bincount(flat,weights=(delta[:,None]*k).ravel(),minlength=n)[:n]
 return np.trunc(boot+lfilter(num,den,stream))
native=sound(rows[:,4]);print('prediction from actual native reads',np.sqrt(np.mean((native-pcm)**2)),flush=True)
results=[]
for nt,bg,sp,pre,zero,early in itertools.product(range(3),range(4,8),range(2,5),range(4,8),[False,True],[False,True]):
 pin=np.zeros((262,341),dtype=bool);upper=False
 for y in range(262):
  active=y<240 or y==261
  for d in range(341):
   if active:
    if zero and y<240 and d==0:upper=bool(a.ctrl&16)
    if 1<=d<=256:
     phase=(d-1)%8
     if phase in [nt,(2+nt)%8]:upper=False
     if phase in [(bg-1)%8,(bg+1)%8]:upper=bool(a.ctrl&16)
    if 257<=d<=320:
     phase=(d-257)%8
     if phase in [nt,(2+nt)%8]:upper=False
     if phase in [sp,(sp+2)%8]:upper=bool(a.ctrl&8)
    if 321<=d<=336:
     phase=(d-321)%8
     if phase in [nt,(2+nt)%8]:upper=False
     if phase in [(pre-1)%8,(pre+1)%8]:upper=bool(a.ctrl&16)
    if d in [337,339]:upper=False
    if early and d in [256,320]:upper=False
   pin[y,d]=upper
 value=np.where(pin[scan,dot],34,17);pred=sound(value);err=pred-pcm;rms=float(np.sqrt(np.mean(err*err)));results.append((rms,nt,bg,sp,pre,zero,early,int(np.max(np.abs(err)))))
for result in sorted(results)[:30]:print(result,flush=True)
