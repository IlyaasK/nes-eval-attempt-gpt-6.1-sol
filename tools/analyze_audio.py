import wave,numpy as np
from scipy.optimize import least_squares
w=wave.open('.out/audio-probe-ref.wav');a=np.frombuffer(w.readframes(w.getnframes()),dtype='<i2').astype(float)
for start in [0,3608]:
 t=np.arange(15,200)
 p=least_squares(lambda p:p[0]*p[2]**t+p[1]*p[3]**t-a[start+t],[10000,-2000,.939235,.987259],xtol=1e-14,gtol=1e-14,ftol=1e-14).x
 print(start,p)
 print(a[start-3 if start else 0:start+15])
# Sum of step tail reflects the effective DAC gain once filters are known.
t=3608
for off in range(-3,4):
 seg=a[t+off:t+off+300].copy()
 # Invert the two known high passes, then examine low pass/antialias step response.
 from scipy.signal import lfilter
 y=lfilter([1,-1],[1,-np.exp(-2*np.pi*90/44100)],np.ones(100))
 b=lfilter([1,-np.exp(-2*np.pi*440/44100)],[np.exp(-2*np.pi*440/44100),-np.exp(-2*np.pi*440/44100)],seg)
 b=lfilter([1,-np.exp(-2*np.pi*90/44100)],[np.exp(-2*np.pi*90/44100),-np.exp(-2*np.pi*90/44100)],b)
 print('inverse',off,b[:12],np.mean(b[30:100]))
