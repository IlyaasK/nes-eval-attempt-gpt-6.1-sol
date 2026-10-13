import wave,numpy as np
from scipy.signal import lfilter
from scipy.optimize import least_squares
w=wave.open('.out/dac-probe-ref.wav');x=np.frombuffer(w.readframes(w.getnframes()),dtype='<i2').astype(float)
r1=np.exp(-2*np.pi*90/44100);r2=np.exp(-2*np.pi*440/44100)
u=lfilter([1,-r2],[r2,-r2],x);u=lfilter([1,-r1],[r1,-r1],u);u=np.diff(u,prepend=0)
amp=32767*(163.67/(24329/141+100)-163.67/(24329/45+100));u/=amp
rows=[line.split() for line in open('.out/dac-events.txt') if line.startswith('DAC')]
ts=np.array([(int(row[1])-8)*44100/(21477272/12) for row in rows]);ds=np.array([1 if int(row[2])==96 else -1 for row in rows]);valid=ts<len(x)-20;ts=ts[valid];ds=ds[valid]
ids=np.floor(ts).astype(int)[:,None]+np.arange(-20,21)[None,:]
valid=(ids>=0)&(ids<len(x));ids0=np.clip(ids,0,len(x)-1)
for window in ['blackman','hann','hamming']:
 def model(p):
  beta,radius,offset,alpha,gain=p
  t=ids-ts[:,None]-offset;s=np.minimum(1,np.abs(t)/radius)
  win={'blackman':lambda:.42+.5*np.cos(np.pi*s)+.08*np.cos(2*np.pi*s),'hann':lambda:.5+.5*np.cos(np.pi*s),'hamming':lambda:(.54+.46*np.cos(np.pi*s))*(s<1)}[window]()
  kernels=beta*np.sinc(beta*t)*win;kernels/=np.sum(kernels,axis=1)[:,None]
  impulse=np.bincount(ids0.ravel(),weights=(kernels*ds[:,None]*valid).ravel(),minlength=len(x))
  return lfilter([alpha],[1,alpha-1],impulse)*gain
 p=least_squares(lambda p:model(p)[200:6000]-u[200:6000],[.92,17,-1.0,.86394,1],bounds=([.7,6,-3,.6,.95],[1.1,20,1,1.1,1.05]),xtol=1e-12,gtol=1e-12,ftol=1e-12,max_nfev=500).x
 print(window,p,'rmse',np.sqrt(np.mean((model(p)[200:6000]-u[200:6000])**2)))
