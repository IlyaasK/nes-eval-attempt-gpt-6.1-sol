"""Forward-model timestamped MIX changes independently of CPU/APU execution."""
import pathlib,re,numpy as np
from scipy.signal import lfilter
R1=.9872590350155737;R2=.9392351762287491;ALPHA=.8639403657579068
DEN=np.convolve(np.convolve([1,-R1],[1,-R2]),[1,ALPHA-1])
B=np.array([1,-1])*R1*R2*ALPHA
def level(p,t):return (95.52/(8128/p+100) if p else 0)+(163.67/(24329/t+100) if t else 0)
def trace_events(path):
 rows=[]
 for line in pathlib.Path(path).read_text().splitlines():
  if line.startswith('MIX '):
   _,tick,p1,p2,tri,noise,dmc=map(lambda s:int(s) if s!='MIX' else s,line.split())
   rows.append((tick-1,level(p1+p2,3*tri+2*noise+dmc)))
 return rows

def predict(events,table,n):
 imp=np.zeros(n+33);old=events[0][1]
 for t,value in events[1:]:
  head=t*44100//1789773-17;frac=(t*44100%1789773)/1789773
  q=min(int(np.float32(frac*256)),255)
  if frac==0:head-=1;q=255
  for j,k in enumerate(table[q]):
   if 0<=head+j<len(imp):imp[head+j]+=(value-old)*k
  old=value
 src=pathlib.Path('src/audio.rs').read_text();cdf0=np.array([float(x) for x in re.search(r'STARTUP:.*?= \[(.*?)\];',src,re.S).group(1).replace(',',' ').split()])
 cdf=np.r_[cdf0,np.ones(n-len(cdf0))]
 boot=lfilter(np.array([1,-2,1])*R1*R2*ALPHA,DEN,cdf)*events[0][1]
 return (lfilter(B,DEN,imp)[:n]+boot)*32767
