"""Original seeded APU register streams with real, cycle-accounted CPU writes.

Without --dmc, replay_noise.py preserves every timestamp. With --dmc the
real CPU program is stalled by DMA, identically in the reference/native. All
pulse, triangle and noise modes are exercised, including very small triangle
periods, sweep overflow, envelopes, frame resets and channel disable/reload.
"""
import argparse,pathlib,random,subprocess,math
p=argparse.ArgumentParser(description=__doc__);p.add_argument('prefix');p.add_argument('--seed',type=lambda x:int(x,0),default=0x2a03);p.add_argument('--events',type=int,default=800);p.add_argument('--dmc',action='store_true');a=p.parse_args()
prefix=pathlib.Path(a.prefix);prefix.parent.mkdir(parents=True,exist_ok=True);rng=random.Random(a.seed)
rows=[(18,0x4017,64),(30,0x4015,15)];clock=100
# All channels start sounding; later streams include independent gates.
initial=[(0x4000,0x3f),(0x4001,0),(0x4002,64),(0x4003,24),(0x4004,0x7f),(0x4005,0),(0x4006,127),(0x4007,24),(0x4008,0xff),(0x400a,84),(0x400b,24),(0x400c,0x3f),(0x400e,3),(0x400f,24)]
if a.dmc:initial +=[(0x4010,79),(0x4011,64),(0x4012,32),(0x4013,1),(0x4015,31)]
for reg,v in initial:rows.append((clock,reg,v));clock+=8
regs=[0x4000,0x4001,0x4002,0x4003,0x4004,0x4005,0x4006,0x4007,0x4008,0x400a,0x400b,0x400c,0x400e,0x400f,0x4015,0x4017]
if a.dmc:regs +=[0x4010,0x4011,0x4012,0x4013]
need_lookup=True
for i in range(a.events):
 previous_clock=clock
 clock+=rng.choice([8,9,13,17,31,63,127,511,rng.randrange(1000,18000)])
 if need_lookup and clock>10000:
  clock=max(clock,previous_clock+1290);need_lookup=False
 reg=rng.choice(regs)
 if reg==0x4015:value=rng.randrange(32 if a.dmc else 16)
 elif reg==0x4011:value=rng.randrange(128)
 elif reg==0x4013:value=rng.choice([0,1,2,3,15,255])
 elif reg==0x4017:value=rng.choice([0,64,128,192])
 elif reg==0x400a:value=rng.choice([0,1,2,3,rng.randrange(256)])
 elif reg==0x400e:value=rng.choice([0,128])+rng.randrange(16)
 else:value=rng.randrange(256)
 rows.append((clock,reg,value))
trace=prefix.with_suffix('.txt');trace.write_text(''.join(f'WRITE {t} {reg:04x} {v}\n' for t,reg,v in rows))
subprocess.run(['python3','tools/replay_noise.py',str(trace),str(prefix.with_suffix('.nes')),'--registers',','.join(f'{r:04x}' for r in regs)],check=True)
frames=math.ceil(clock/(1789773/60.0988))+3;prefix.with_suffix('.frames').write_text(str(frames));print('frames',frames,'last',clock)
