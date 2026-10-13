import subprocess,os,numpy as np,wave
w=wave.open('.out/ref-noise.wav');b=np.frombuffer(w.readframes(w.getnframes()),dtype='<i2').astype(float)
seed=1;best=[]
for step in range(18):
 for timer in range(4):
  e=dict(os.environ,NES_NOISE_SEED=str(seed),NES_NOISE_TIMER=str(timer),LAST_ONLY='1')
  subprocess.run(['target/release/examples/run','.out/noise.nes','20','.out/noise-fit'],env=e,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True)
  a=np.fromfile('.out/noise-fit/audio.raw',dtype='<i2').astype(float);n=min(len(a),len(b));r=np.sqrt(np.mean((a[100:n]-b[100:n])**2));best.append((r,step,seed,timer))
 seed=(seed>>1)|(((seed^(seed>>1))&1)<<14)
print(sorted(best)[:12])
