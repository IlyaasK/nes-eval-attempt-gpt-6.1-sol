"""Reference lookup phase adjustment for timestamped CPU writes.

A write at cycle t changes the APU mix at t+1. The lookup uses the phase
at t, before the reset-clock offset. Quantizing the integer rational avoids
losing the rare whole-sample and immediately-below-boundary cases.
"""
import numpy as np

def adjustment(cycles):
    cycles=np.asarray(cycles,dtype=np.int64)
    fraction=((cycles*44100)%1789773).astype(float)/1789773
    scaled=(fraction*256).astype(np.float32)
    phase=np.minimum(np.floor(scaled),255).astype(float)/256
    phase=np.where(fraction==0,-1/256,phase)
    return phase-fraction+1/512
