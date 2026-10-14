"""Build original DAC impulses immediately below all 256 phase boundaries.

The clock ratio reduces to14700/596591. Modular inversion places each
write at the closest representable CPU cycle below its desired phase.
Long gaps isolate the filter response; this probes numerical phase rules
without relying on a game's APU sequencing or cartridge implementation.
"""
import argparse,math,pathlib,subprocess
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('prefix');a=p.parse_args()
prefix=pathlib.Path(a.prefix);prefix.parent.mkdir(parents=True,exist_ok=True)
period=596591;inverse=pow(14700,-1,period);last=10000;rows=[];phases=[]
for phase in range(256):
 numerator=phase*period//256
 clock=(numerator*inverse)%period
 clock+=max(0,math.ceil((last+3500-clock)/period))*period
 rows.extend([f'WRITE {clock} 4011 96',f'WRITE {clock+1700} 4011 0'])
 phases.append(f'{phase} {clock}')
 last=clock+1700
trace=prefix.with_suffix('.txt');trace.write_text('\n'.join(rows)+'\n')
prefix.with_suffix('.phases').write_text('\n'.join(phases)+'\n')
subprocess.run(['python3','tools/replay_noise.py',str(trace),str(prefix.with_suffix('.nes')),
 '--registers','4011','--max-cycles',str(last+1)],check=True)
print('frames',math.ceil(last/(1789773/60.0988))+3,'last write',last)
