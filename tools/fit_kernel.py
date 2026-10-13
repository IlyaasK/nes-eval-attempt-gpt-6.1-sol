import wave,numpy as np
from scipy.signal import lfilter
from scipy.optimize import least_squares
w=wave.open('.out/audio-probe-ref.wav');x=np.frombuffer(w.readframes(w.getnframes()),dtype='<i2').astype(float)
r1=np.exp(-2*np.pi*90/44100);r2=np.exp(-2*np.pi*440/44100)
u=lfilter([1,-r2],[r2,-r2],x);u=lfilter([1,-r1],[r1,-r1],u)
amp=32767*(163.67/(24329/141+100)-163.67/(24329/45+100));u=np.diff(u[3592:3630])/amp
for kind in ['blackman','hann','hamming']:
 def impulse(p):
  beta,phase,radius,alpha,gain=p
  t=np.arange(-16,21)-phase
  s=np.minimum(1,np.abs(t)/radius)
  win={'blackman':lambda:.42+.5*np.cos(np.pi*s)+.08*np.cos(2*np.pi*s),'hann':lambda:.5+.5*np.cos(np.pi*s),'hamming':lambda:(.54+.46*np.cos(np.pi*s))*(s<1)}[kind]()
  h=beta*np.sinc(beta*t)*win;h=h/h.sum()*gain
  return lfilter([alpha],[1,alpha-1],h)
 p=least_squares(lambda p:impulse(p)-u,[.93,.1,17,.86,1],bounds=([.5,-.8,6,.4,.95],[1.2,.8,25,1.5,1.05]),xtol=1e-14,gtol=1e-14,ftol=1e-14,max_nfev=3000).x
 print(kind,p,'rms',np.sqrt(np.mean((impulse(p)-u)**2)))
 print(np.round(impulse(p)[10:25],6));print(np.round(u[10:25],6))
