"""Recover a generic reconstruction FIR from an original isolated DAC ROM.

Inputs are reference WAV, timestamped WRITE trace, and a reference WAV from
power_audio_probe.s. The DAC ROM must leave triangle at its power value 15,
leave other channels off, and space writes far enough to isolate impulses.
The output is a Rust source file, independent of the calibration cartridge.

First recover a spline from inverse-filtered impulses, holding out half the
input phases. Then refine in PCM space so quantization noise is not amplified
by the inverse filters. Validate generated coefficients with fresh patterns
and game replays before adopting them.
"""
import argparse,pathlib,wave
import numpy as np
from scipy.signal import lfilter
from scipy.interpolate import BSpline
from scipy.sparse import vstack,diags,csr_matrix
from scipy.sparse.linalg import lsqr,LinearOperator
from audio_phase import adjustment
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('wav');p.add_argument('trace');p.add_argument('power_wav');p.add_argument('output')
p.add_argument('--iterations',type=int,default=300)
args=p.parse_args()
def samples(path):
 with wave.open(path) as w:
  assert w.getframerate()==44100 and w.getsampwidth()==2 and w.getnchannels()==1
  return np.frombuffer(w.readframes(w.getnframes()),'<i2').astype(float)
def mix(dac):return 32767*163.67/(24329/(45+dac)+100)
pcm=samples(args.wav);decoded=pcm+np.sign(pcm)*.5;n=len(pcm)
r=44100/1789773;r1=np.exp(-2*np.pi*90/44100);r2=np.exp(-2*np.pi*440/44100);alpha=1-np.exp(-2*np.pi*14000/44100)
rows=[];previous=0
for line in pathlib.Path(args.trace).read_text().splitlines():
 words=line.split()
 if len(words)==4 and words[0]=='WRITE' and int(words[2],16)==0x4011:
  cycle,value=int(words[1]),int(words[3])&127;delta=mix(value)-mix(previous);previous=value;t=(cycle-8)*r
  if delta and 32<t<n-50:rows.append((t,delta,cycle))
ts=np.array([a for a,b,c in rows]);amps=np.array([b for a,b,c in rows]);scale=np.max(np.abs(amps));quant=adjustment([c for a,b,c in rows])
u=lfilter([1,-r2],[r2,-r2],decoded);u=lfilter([1,-r1],[r1,-r1],u);u=np.diff(u,prepend=0)
ids=np.floor(ts).astype(int)[:,None]+np.arange(-28,36)[None,:]
values=u[ids]/amps[:,None];values-=values[:,np.r_[0:7,59:64]].mean(axis=1)[:,None]
true_kernel=(values-np.c_[np.zeros(len(values)),values[:,:-1]]*(1-alpha))/alpha
x=ids-ts[:,None]-quant[:,None]+1.30936
knots=np.r_[[-18]*3,np.arange(-18,18+1/512,1/256),[18]*3];nc=len(knots)-4
train=np.arange(len(ts))%2==0
mat=BSpline.design_matrix(np.clip(x[train].ravel(),-18,18),knots,3).tocsr()
d2=diags([np.ones(nc-2),-2*np.ones(nc-2),np.ones(nc-2)],[0,1,2],shape=(nc-2,nc)).tocsr()
prior=lsqr(vstack([mat,d2*.01]),np.r_[true_kernel[train].ravel(),np.zeros(nc-2)],atol=1e-10,btol=1e-10,iter_lim=300)[0]
hold=BSpline.design_matrix(np.clip(x[~train].ravel(),-18,18),knots,3)@prior
hold=lfilter([alpha],[1,alpha-1],hold.reshape(values[~train].shape),axis=1)
print('held-out inverse-filter residual PCM',np.sqrt(np.mean((hold-values[~train])**2))*scale,flush=True)
ids=np.floor(ts).astype(int)[:,None]+np.arange(-19,20)[None,:];x=ids-ts[:,None]-quant[:,None]+1.30936
basis=BSpline.design_matrix(np.clip(x.ravel(),-18,18),knots,3).tocsr();basis=basis.multiply((np.abs(x.ravel())<16)[:,None]).tocsr()
weights=np.broadcast_to((amps/scale)[:,None],ids.shape).ravel();flat=ids.ravel()
a=np.convolve(np.convolve([1,-r1],[1,-r2]),[1,alpha-1]);b=np.array([1,-1])*r1*r2*alpha
power=samples(args.power_wav);power+=np.sign(power)*.5
power=lfilter([1,alpha-1],[alpha],power);power=lfilter([1,-r2],[r2,-r2],power);power=lfilter([1,-r1],[r1,-r1],power)
cdf=np.r_[power[:16]/mix(0),np.ones(n-16)]
boot=lfilter(np.array([1,-2,1])*r1*r2*alpha,a,cdf)*mix(0);observed=(decoded-boot)/scale
def audio(c):return lfilter(b,a,np.bincount(flat,weights=(basis@c)*weights,minlength=n))
sums=csr_matrix((np.ones(len(flat)),(np.repeat(np.arange(len(ts)),ids.shape[1]),np.arange(len(flat)))),shape=(len(ts),len(flat)))
dc=(sums@basis).tocsr();dw=10.;sw=.001
def forward(c):return np.r_[audio(c),dw*(dc@c),sw*(d2@c)]
def adjoint(y):
 v=lfilter(b,a,y[:n][::-1])[::-1]
 return basis.T@(v[flat]*weights)+dw*(dc.T@y[n:n+len(ts)])+sw*(d2.T@y[n+len(ts):])
op=LinearOperator((n+len(ts)+nc-2,nc),matvec=forward,rmatvec=adjoint)
target=np.r_[observed-audio(prior),dw*(1-dc@prior),np.zeros(nc-2)]
coeff=prior+lsqr(op,target,atol=1e-9,btol=1e-9,iter_lim=args.iterations)[0]
print('PCM refinement residual',np.sqrt(np.mean((observed-audio(coeff))**2))*scale,flush=True)
text='''//! Generic reconstruction filter calibrated with isolated DAC impulses.
//! Uniform cubic B-spline coefficients, spacing 1/256 sample, origin -18.
//! Reproduce with tools/calibrate_audio.py; validate with fresh input phases
//! and independent game audio. This filter is independent of cartridge data.
'''+f'const IMPULSE_COEFFS: [f64; {len(coeff)}] = [\n'
text+=''.join('    '+', '.join(f'{c:.14e}' for c in coeff[i:i+4])+',\n' for i in range(0,len(coeff),4))+'];\n'
text+='''pub(super) fn impulse(x: f64) -> f64 {
    if x.abs() >= 16.0 { return 0.0; }
    let position = (x + 18.0) * 256.0;
    let i = position.floor() as usize;
    let t = position - i as f64;
    let t2 = t * t;
    let t3 = t2 * t;
    let weights = [(1.0 - t).powi(3) / 6.0,
        (3.0 * t3 - 6.0 * t2 + 4.0) / 6.0,
        (-3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0) / 6.0,
        t3 / 6.0];
    let mut result = 0.0;
    for j in 0..4 { result += IMPULSE_COEFFS[i + j] * weights[j]; }
    result
}
'''
pathlib.Path(args.output).write_text(text)
