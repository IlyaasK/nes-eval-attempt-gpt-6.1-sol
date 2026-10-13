"""Measure reference DAC-marker write times by exact synthetic CPU replays.

Given a native trace containing WRITE <cycle> 4011 <value> and the oracle
WAV for that diagnostic ROM, replay every DAC write at several cycle offsets.
The closest local waveform identifies the reference timestamp without making
assumptions about its resampler. Reports transitions whose reference phase is
covered by the requested offset range; inspect RMSE before trusting a match.
"""
import argparse
import json
import pathlib
import subprocess
import sys
import wave
import numpy as np

p = argparse.ArgumentParser(description=__doc__)
p.add_argument('trace')
p.add_argument('reference')
p.add_argument('frames', type=int)
p.add_argument('output')
p.add_argument('--radius', type=int, default=5)
a = p.parse_args()
out = pathlib.Path(a.output)
out.mkdir(parents=True, exist_ok=True)


def read_wav(path):
    with wave.open(str(path)) as w:
        assert w.getframerate() == 44100 and w.getnchannels() == 1
        return np.frombuffer(w.readframes(w.getnframes()), '<i2').astype(float)


reference = read_wav(a.reference)
candidates = []
offsets = list(range(-a.radius, a.radius + 1))
for offset in offsets:
    rom = out / f'replay_{offset}.nes'
    wav = out / f'replay_{offset}.wav'
    if not wav.exists():
        subprocess.run([sys.executable, 'tools/replay_noise.py', a.trace,
                        str(rom), '--registers', '4011,4017', '--offset',
                        str(offset)], check=True, stdout=subprocess.DEVNULL)
        subprocess.run(['oracle', 'run', str(rom), str(a.frames),
                        '--dump-audio', str(wav)], check=True,
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    candidates.append(read_wav(wav))

last = 0
results = []
for line in pathlib.Path(a.trace).read_text().splitlines():
    row = line.split()
    if row[:1] != ['WRITE'] or row[2] != '4011':
        continue
    cycle, value = int(row[1]), int(row[3]) & 127
    if value == last:
        continue
    last = value
    center = int(cycle * 44100 / 1789773 - 1.334)
    low, high = center - 5, center + 20
    if low < 0 or high >= min(len(reference), *(len(c) for c in candidates)):
        continue
    scores = [float(np.sqrt(np.mean((c[low:high] - reference[low:high]) ** 2)))
              for c in candidates]
    index = int(np.argmin(scores))
    result = dict(native_cycle=cycle, reference_cycle=cycle + offsets[index],
                  offset=offsets[index], value=value, rmse=scores[index])
    results.append(result)
    print(json.dumps(result), flush=True)
(out / 'timing.json').write_text(json.dumps(results, indent=2))
