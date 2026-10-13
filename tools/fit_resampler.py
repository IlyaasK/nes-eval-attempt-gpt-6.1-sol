import wave,numpy as np
from scipy.signal import lfilter
from scipy.optimize import least_squares
from scipy.integrate import cumulative_trapezoid
from scipy.special import i0
w=wave.open('.out/audio-probe-ref.wav');x=np.frombuffer(w.readframes(w.getnframes()),dtype='<i2').astype(float)
r1=np.exp(-2*np.pi*90/44100);r2=np.exp(-2*np.pi*440/44100);r3=np.exp(-2*np.pi*14000/44100)
u=lfilter([1,-r3],[1-r3],x);u=lfilter([1,-r2],[r2,-r2],u);u=lfilter([1,-r1],[r1,-r1],u)
u=u[:14]/(163.67/(24329/45+100)*32767)
for kind in ['blackman','hann','hamming','kaiser']:
 def step(p):
  cutoff,radius,offset,beta=p
  t=np.linspace(-radius,radius,20000)
  s=t/radius
  window={'blackman':lambda:.42+.5*np.cos(np.pi*s)+.08*np.cos(2*np.pi*s),'hann':lambda:.5+.5*np.cos(np.pi*s),'hamming':lambda:.54+.46*np.cos(np.pi*s),'kaiser':lambda:i0(beta*np.sqrt(np.maximum(0,1-s*s)))/i0(beta)}[kind]()
  a=cumulative_trapezoid(cutoff*np.sinc(cutoff*t)*window,t,initial=0)
  return np.interp(np.arange(14)+offset,t,a)/a[-1]
 p=least_squares(lambda p:step(p)-u,[.95,17.5,.53,6],bounds=([.8,8,.3,1],[1.,30,.8,14]),diff_step=1e-5,max_nfev=500).x
 print(kind,p,np.sqrt(np.mean((step(p)-u)**2)),np.round(step(p),6))
print('reference',np.round(u,6))
