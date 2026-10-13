//! Band-limited delta resampling. The source clock is the NTSC CPU clock;
//! the output clock is 44.1 kHz. Keeping the clock fraction avoids frame drift.
const PHASES: usize = 256;
const TAPS: usize = 34;
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
        let mut kernels = vec![[0.0;TAPS];PHASES+1];
        for (p, kernel) in kernels.iter_mut().enumerate() {
            let frac = p as f64 / PHASES as f64;
            let mut sum = 0.0;
            for (i, k) in kernel.iter_mut().enumerate() {
                let x = i as f64 - 16.0 - frac;
                let w =
                    if x.abs() < 16.0 {
                        0.42 + 0.5 * (std::f64::consts::PI * x / 16.0).cos() +
                            0.08 * (2.0 * std::f64::consts::PI * x / 16.0).cos()
                    } else { 0.0 };
                let z = std::f64::consts::PI * x * 0.9206;
                *k =
                    if z.abs() < 1e-12 {
                        0.9206 * w
                    } else { 0.9206 * w * z.sin() / z };
                sum += *k;
            }
            for k in kernel { *k /= sum; }
        }
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
            let clock =
                (self.phase + 44100 * 12) as f64 / 21477272.0 -
                    (8.0 * 44100.0 * 12.0 / 21477272.0);
            let center =
                17.0 + clock - if self.started { 1.334 } else { 0.0 };
            self.started = true;
            let base = center.floor() as isize - 16;
            let pos = (center - center.floor()) * PHASES as f64;
            let p = pos as usize;
            let f = pos - p as f64;
            for i in 0..TAPS {
                let offset = base + i as isize;
                if offset >= 0 {
                    let k =
                        self.kernels[p][i] +
                            f * (self.kernels[p + 1][i] - self.kernels[p][i]);
                    self.ring[(self.head + offset as usize) & 127] += delta * k;
                }
            }
        }
        self.phase += 44100 * 12;
        if self.phase < 21477272 { return None }
        self.phase -= 21477272;
        self.integrator += self.ring[self.head];
        self.ring[self.head] = 0.0;
        self.head = (self.head + 1) & 127;
        self.emitted += 1;
        if self.emitted <= 17 { return None }
        let x = self.integrator;
        let y = 0.987259035 * self.hp1_y + 0.987259035 * (x - self.hp1_x);
        self.hp1_x = x;
        self.hp1_y = y;
        let z = 0.939235180 * self.hp2_y + 0.939235180 * (y - self.hp2_x);
        self.hp2_x = y;
        self.hp2_y = z;
        self.lp += 0.863940 * (z - self.lp);
        Some((self.lp * 32767.0).clamp(-32767.0, 32767.0) as i16)
    }
}
