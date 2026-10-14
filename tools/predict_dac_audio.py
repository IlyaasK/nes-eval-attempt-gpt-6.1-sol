"""Forward-model an original DAC trace using a candidate generic FIR table."""
import argparse,pathlib,wave,re
import numpy as np
from scipy.signal import lfilter
p=argparse.ArgumentParser(description=__doc__);p.add_argument('table');p.add_argument('trace');p.add_argument('reference');p.add_argument('--key',default='tables');p.add_argument('--native');a=p.parse_args()
K=np.load(a.table)[a.key];N=0
with wave.open(a.reference) as w:
 ref=np.frombuffer(w.readframes(w.getnframes()),'<i2');N=len(ref)
def mix(v):return 163.67/(24329/(45+v)+100)
imp=np.zeros(N+33);prev=0
for line in pathlib.Path(a.trace).read_text().splitlines():
 words=line.split()
 if len(words)!=4 or words[0]!='WRITE' or int(words[2],16)!=0x4011:continue
 t,v=int(words[1]),int(words[3])&127
 head=t*44100//1789773-17;frac=(t*44100%1789773)/1789773
 q=min(int(np.floor(np.float32(frac*256))),255)
 if frac==0:head-=1;q=255
 for i,k in enumerate(K[q]):
  if 0<=head+i<len(imp):imp[head+i]+=(mix(v)-mix(prev))*k
 prev=v
r1=.9872590350155737;r2=.9392351762287491;alpha=.8639403657579068
afilter=np.convolve(np.convolve([1,-r1],[1,-r2]),[1,alpha-1])
src=pathlib.Path('src/audio.rs').read_text();startup=np.array([float(x) for x in re.search(r'STARTUP:.*?= \[(.*?)\];',src,re.S).group(1).replace(',',' ').split()])
cdf=np.r_[startup,np.ones(N-len(startup))]
boot=lfilter(np.array([1,-2,1])*r1*r2*alpha,afilter,cdf)*mix(0)
y=lfilter(np.array([1,-1])*r1*r2*alpha,afilter,imp)[:N]+boot
pcm=np.clip(np.trunc(y*32767),-32767,32767).astype('<i2')
for name,data in [('oracle',ref),('native',np.fromfile(a.native,'<i2') if a.native else None)]:
 if data is not None:
  err=pcm.astype(float)-data[:N];print(name,'len',len(data),'rms',np.sqrt(np.mean(err**2)),'peak',np.max(np.abs(err)),'different',np.count_nonzero(err))
