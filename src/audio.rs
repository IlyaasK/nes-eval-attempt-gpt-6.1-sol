//! Band-limited delta resampling. The source clock is the NTSC CPU clock;
//! the output clock is 44.1 kHz. Keeping the clock fraction avoids frame drift.
const CPU_HZ: u64 = 1_789_773;
const CLOCK_SCALE: u64 = CPU_HZ * 12;
const PHASES: usize = 256;
const TAPS: usize = 33;
#[path = "audio_phase_table.rs"]
mod kernel;
pub struct Resampler {
    kernels: Vec<[f64; TAPS]>,
    ring: [f64; 128],
    head: usize,
    integrator: f64,
    last: f64,
    phase: u64,
    emitted: u64,
    started: bool,
    hp1_x: f64,
    hp1_y: f64,
    hp2_x: f64,
    hp2_y: f64,
    lp: f64,
}
impl Resampler {
    pub fn new() -> Self {
        let kernels = kernel::TABLES.to_vec();
        Self {
            kernels,
            ring: [0.0; 128],
            head: 0,
            integrator: 0.0,
            last: 0.0,
            phase: 8 * 44100 * 12,
            emitted: 0,
            started: false,
            hp1_x: 0.0,
            hp1_y: 0.0,
            hp2_x: 0.0,
            hp2_y: 0.0,
            lp: 0.0,
        }
    }
    pub fn clock(&mut self, level: f64) -> Option<i16> {
        let delta = level - self.last;
        if delta != 0.0 {
            self.last = level;
            if !self.started {
                // Fixed power-on clock phase. These are the cumulative taps
                // of the startup impulse after the 17-sample output delay.
                const STARTUP: [f64; 17] = [
                    0.938755072726, 1.031980775353, 0.984252702826,
                    1.006131841473, 0.999401487416, 0.997925765493,
                    1.002991013871, 0.997121668659, 1.002222053117,
                    0.998539525466, 1.000865880595, 0.999587988277,
                    1.000071909274, 0.999928906261, 0.999984257325,
                    1.000027795906, 1.0,
                ];
                let mut previous = 0.0;
                for (i, &cumulative) in STARTUP.iter().enumerate() {
                    self.ring[(self.head + 17 + i) & 127] +=
                        delta * (cumulative - previous);
                    previous = cumulative;
                }
                self.started = true;
            } else {
                // Use the absolute CPU phase, including reset cycles. The table
                // lookup rounds its scaled fractional phase to f32 first;
                // this matters at representable cycles just below a phase
                // boundary. At an exact whole sample the reference's clock
                // conversion approaches from below, selecting phase255 of
                // the preceding sample. Original 256-phase DAC probes
                // protect both numerical edge cases independently of APU.
                let quantized = Self::quantized_phase(self.phase);
                let p = if self.phase == 0 { 255 } else { (quantized * 256.0) as usize };
                let base = if self.phase == 0 { -1isize } else { 0isize };
                for i in 0..TAPS {
                    let offset = base + i as isize;
                    if offset >= 0 {
                        self.ring[(self.head + offset as usize) & 127] += delta * self.kernels[p][i];
                    }
                }
            }
        }
        self.phase += 44100 * 12;
        if self.phase < CLOCK_SCALE { return None }
        self.phase -= CLOCK_SCALE;
        self.integrator += self.ring[self.head];
        self.ring[self.head] = 0.0;
        self.head = (self.head + 1) & 127;
        self.emitted += 1;
        if self.emitted <= 17 { return None }
        let x = self.integrator;
        let y = 0.9872590350155737 * self.hp1_y + 0.9872590350155737 * (x - self.hp1_x);
        self.hp1_x = x;
        self.hp1_y = y;
        let z = 0.9392351762287491 * self.hp2_y + 0.9392351762287491 * (y - self.hp2_x);
        self.hp2_x = y;
        self.hp2_y = z;
        self.lp += 0.8639403657579068 * (z - self.lp);
        Some((self.lp * 32767.0).clamp(-32767.0, 32767.0) as i16)
    }
    fn quantized_phase(phase: u64) -> f64 {
        if phase == 0 { return -1.0 / PHASES as f64; }
        let scaled = (phase as f64 * PHASES as f64 / CLOCK_SCALE as f64) as f32;
        scaled.floor().min((PHASES - 1) as f32) as f64 / PHASES as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn phase_lookup_matches_whole_samples_and_nearby_fractional_boundaries() {
        assert_eq!(Resampler::quantized_phase(0), -1.0 / 256.0);
        // For these cycles the true fraction is just below143/256 and
        //173/256, but the lookup rounds to the boundary in single precision.
        for (cycle, phase) in [(36255223u64, 143.0), (43737250, 173.0)] {
            let rem = (cycle * 44100 * 12) % CLOCK_SCALE;
            assert!((rem as f64 / CLOCK_SCALE as f64) < phase / 256.0);
            assert_eq!(Resampler::quantized_phase(rem), phase / 256.0);
        }
        // Other nearby boundaries retain the preceding phase.
        for (cycle, phase) in [(8078618u64, 30.0), (15560645, 60.0)] {
            let rem = (cycle * 44100 * 12) % CLOCK_SCALE;
            assert_eq!(Resampler::quantized_phase(rem), (phase - 1.0) / 256.0);
        }
    }
}
