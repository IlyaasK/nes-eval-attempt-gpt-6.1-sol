"""Fit a generic 256-phase reconstruction FIR in PCM space.

Training uses isolated DAC impulses at multiple levels. Separate DAC levels
are held out, and no cartridge-specific behavior is included. Output is an
experimental table; adoption requires independent patterns/game validation.
"""
import argparse,pathlib,re,wave
import numpy as np
from scipy.signal import lfilter
from scipy.sparse.linalg import LinearOperator,lsqr
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('root');p.add_argument('output');p.add_argument('--iterations',type=int,default=250)
p.add_argument('--smooth',type=float,default=.02);p.add_argument('--ridge',type=float,default=.002)
a=p.parse_args();root=pathlib.Path(a.root);N=384;T=33
r=44100/1789773;r1=.9872590350155737;r2=.9392351762287491;alpha=.8639403657579068
b=np.array([1.,-1.])*r1*r2*alpha;den=np.convolve(np.convolve([1,-r1],[1,-r2]),[1,alpha-1])
g=lfilter(b,den,np.r_[1.,np.zeros(N-1)])
B=np.zeros((N,T))
for j in range(T):B[j:,j]=g[:N-j]
center=17-1.334-7*r+1/512
src=pathlib.Path('src/audio_kernel.rs').read_text();coeff=np.array([float(x) for x in re.search(r'IMPULSE_COEFFS:.*?= \[(.*?)\];',src,re.S).group(1).replace(',',' ').split()])
def impulse(x):
 x=np.asarray(x);pos=(x+18)*256;i=np.floor(pos).astype(int);t=pos-i
 w=np.array([(1-t)**3,3*t**3-6*t*t+4,-3*t**3+3*t*t+3*t+1,t**3]).T/6
 return np.sum(coeff[i[:,None]+np.arange(4)]*w,axis=1)*(np.abs(x)<16)
source=[]
for q in range(257):
 row=impulse(np.arange(34)-16-q/256);source.append(row/row.sum())
source=np.array(source);prior=np.zeros((256,T))
for q in range(256):
 c=center+q/256;base=int(np.floor(c))-16;pos=(c-np.floor(c))*256;i=int(pos);f=pos-i
 row=source[i]*(1-f)+source[i+1]*f
 for j,k in enumerate(row):
  if 0<=base+j<T:prior[q,base+j]=k
cycles=np.loadtxt(root/'dac96.phases',dtype=np.int64)[:,1]
heads=cycles*44100//1789773-17
fall=(cycles+1700)*44100//1789773-17
q=((cycles+1700)*44100%1789773/1789773*256).astype(np.float32).astype(int)
shift=fall-heads
assert len(np.unique(q))==256
levels=[16,64,96,127];waves={};gamma={}
def mix(v):return 32767*163.67/(24329/(45+v)+100)
for v in levels:
 with wave.open(str(root/f'dac{v}.wav')) as w:
  pcm=np.frombuffer(w.readframes(w.getnframes()),'<i2').astype(float)
 waves[v]=pcm[heads[:,None]+np.arange(N)]
 gamma[v]=mix(v)-mix(0)
train=[96,127];weight=sum(gamma[v]**2 for v in train)
target=sum(gamma[v]*(waves[v]+np.sign(waves[v])*.5) for v in train)/weight
# Numerical adjoint uses the inverse phase permutation for the falling edge.
def forward_audio(k):
 y=k@B.T
 for d in np.unique(shift):
  ids=np.flatnonzero(shift==d);y[ids,d:]-=(k[q[ids]]@B[:N-d].T)
 return y

def adj_audio(y):
 k=y@B
 for d in np.unique(shift):
  ids=np.flatnonzero(shift==d);k[q[ids]]-=y[ids,d:]@B[:N-d]
 return k
# Interior smoothness prior only: the wrap boundary shifts taps by a sample.
# Weak correction regularization keeps poorly constrained stop-band taps stable.
dcw=100.;sw=a.smooth;rw=a.ridge
mask=(np.abs(np.arange(T)[None,:]-(center+np.arange(256)[:,None]/256))<16)
def smooth(k):
 prev=np.roll(k,1,axis=0);nxt=np.roll(k,-1,axis=0)
 prev[0,:-1]=k[-1,1:];prev[0,-1]=0
 nxt[-1,1:]=k[0,:-1];nxt[-1,0]=0
 return prev+nxt-2*k
shape=(256*N+256+256*T+256*T,256*T)
def forward(x):
 k=x.reshape(256,T)*mask
 return np.r_[forward_audio(k).ravel(),dcw*k.sum(axis=1),sw*smooth(k).ravel(),rw*x]
def adjoint(y):
 ix=256*N;k=adj_audio(y[:ix].reshape(256,N));k+=dcw*y[ix:ix+256,None];ix+=256
 k+=sw*smooth(y[ix:ix+256*T].reshape(256,T));ix+=256*T
 return (k*mask).ravel()+rw*y[ix:]
op=LinearOperator(shape,matvec=forward,rmatvec=adjoint)
def report(k,label):
 y=forward_audio(k)
 for v in levels:
  pred=np.trunc(y*gamma[v]);err=pred-waves[v]
  print(label,v,'rms',np.sqrt(np.mean(err**2)),'peak',np.max(np.abs(err)),'float',np.sqrt(np.mean((y*gamma[v]-waves[v]-np.sign(waves[v])*.5)**2)),flush=True)
report(prior,'prior')
res=target-forward_audio(prior)
rhs=np.r_[res.ravel(),dcw*(1-prior.sum(axis=1)),np.zeros(256*T),np.zeros(256*T)]
fit=lsqr(op,rhs,atol=1e-10,btol=1e-10,iter_lim=a.iterations,show=True)
k=prior+fit[0].reshape(256,T)*mask
# Exact unity DC preserves the integrator for arbitrary infinite streams.
k/=k.sum(axis=1)[:,None]
report(k,'fit');print('delta max',np.abs(k-prior).max(),'rms',np.sqrt(np.mean((k-prior)**2)),flush=True)
np.savez(a.output,tables=k,prior=prior,cycles=cycles,heads=heads,q=q,shift=shift)
k.astype('<f8').tofile(str(a.output)+'.bin')
