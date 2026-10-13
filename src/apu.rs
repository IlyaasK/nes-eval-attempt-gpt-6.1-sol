const LEN: [u8; 32] =
    [10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24,
            18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30];
const NOISE: [u16; 16] =
    [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034,
            4068];
const DMC: [u16; 16] =
    [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72,
            54];
#[derive(Default)]
struct Envelope {
    start: bool,
    divider: u8,
    decay: u8,
}
impl Envelope {
    fn clock(&mut self, r: u8) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = r & 15
        } else if self.divider > 0 {
            self.divider -= 1
        } else {
            self.divider = r & 15;
            if self.decay > 0 {
                self.decay -= 1
            } else if r & 32 != 0 { self.decay = 15 }
        }
    }
    fn output(&self, r: u8) -> u8 {
        if r & 16 != 0 { r & 15 } else { self.decay }
    }
}
#[derive(Default)]
struct Pulse {
    r: [u8; 4],
    enable: bool,
    length: u8,
    period: u16,
    timer: u16,
    seq: u8,
    env: Envelope,
    sweep_div: u8,
    sweep_reload: bool,
}
impl Pulse {
    fn target(&self, one: bool) -> i32 {
        let shift = self.r[1] & 7;
        let delta = self.period >> shift;
        if self.r[1] & 8 != 0 {
            self.period as i32 - delta as i32 - if one { 1 } else { 0 }
        } else { self.period as i32 + delta as i32 }
    }
    fn output(&self, one: bool) -> u8 {
        const DUTY: [u8; 4] =
            [0b01000000, 0b01100000, 0b01111000, 0b10011111];
        if self.length == 0 || self.period < 8 || self.target(one) > 2047 ||
                (DUTY[(self.r[0] >> 6) as usize] >> (7 - self.seq)) & 1 == 0 {
            0
        } else { self.env.output(self.r[0]) }
    }
    fn timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            self.seq = (self.seq + 1) & 7
        } else { self.timer -= 1 }
    }
    fn half(&mut self, one: bool) {
        if self.r[0] & 32 == 0 && self.length > 0 { self.length -= 1 }
        let target = self.target(one);
        if self.sweep_div == 0 && self.r[1] & 128 != 0 && self.r[1] & 7 != 0
                    && self.period >= 8 && (0..=2047).contains(&target) {
            self.period = target as u16
        }
        if self.sweep_div == 0 || self.sweep_reload {
            self.sweep_div = (self.r[1] >> 4) & 7;
            self.sweep_reload = false
        } else { self.sweep_div -= 1 }
    }
    fn write(&mut self, i: usize, v: u8) {
        self.r[i] = v;
        match i {
            1 => self.sweep_reload = true,
            2 => self.period = (self.period & 0x700) | v as u16,
            3 => {
                self.period = (self.period & 255) | ((v as u16 & 7) << 8);
                if self.enable { self.length = LEN[(v >> 3) as usize] }
                self.seq = 0;
                self.env.start = true
            }
            _ => {}
        }
    }
}
#[derive(Default)]
struct Triangle {
    r: [u8; 4],
    enable: bool,
    length: u8,
    period: u16,
    timer: u16,
    seq: u8,
    linear: u8,
    reload: bool,
}
impl Triangle {
    fn quarter(&mut self) {
        if self.reload {
            self.linear = self.r[0] & 127
        } else if self.linear > 0 { self.linear -= 1 }
        if self.r[0] & 128 == 0 { self.reload = false }
    }
    fn output(&self) -> u8 {
        if self.seq < 16 { 15 - self.seq } else { self.seq - 16 }
    }
    fn timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            if self.length > 0 && self.linear > 0 {
                self.seq = (self.seq + 1) & 31
            }
        } else { self.timer -= 1 }
    }
    fn write(&mut self, i: usize, v: u8) {
        self.r[i] = v;
        match i {
            2 => self.period = (self.period & 0x700) | v as u16,
            3 => {
                self.period = (self.period & 255) | ((v as u16 & 7) << 8);
                if self.enable { self.length = LEN[(v >> 3) as usize] }
                self.reload = true
            }
            _ => {}
        }
    }
}
struct Noise {
    r: [u8; 4],
    enable: bool,
    length: u8,
    timer: u16,
    shift: u16,
    env: Envelope,
}
impl Default for Noise {
    fn default() -> Self {
        Self {
            r: [0; 4],
            enable: false,
            length: 0,
            timer: 1,
            shift: 0x4000,
            env: Envelope::default(),
        }
    }
}
impl Noise {
    fn output(&self) -> u8 {
        if self.length > 0 && self.shift & 1 == 0 {
            self.env.output(self.r[0])
        } else { 0 }
    }
    fn timer(&mut self) {
        if self.timer == 0 {
            self.timer = NOISE[(self.r[2] & 15) as usize];
            let bit =
                (self.shift ^
                            (self.shift >> if self.r[2] & 128 != 0 { 6 } else { 1 })) &
                    1;
            self.shift = (self.shift >> 1) | (bit << 14)
        } else { self.timer -= 1 }
    }
    fn write(&mut self, i: usize, v: u8) {
        self.r[i] = v;
        if i == 3 {
            if self.enable { self.length = LEN[(v >> 3) as usize] }
            self.env.start = true
        }
    }
}
pub struct Apu {
    pulse: [Pulse; 2],
    tri: Triangle,
    noise: Noise,
    dmc_r: [u8; 4],
    pub dmc_addr: u16,
    pub dmc_left: u16,
    dmc_buffer: Option<u8>,
    dmc_shift: u8,
    dmc_bits: u8,
    dmc_silent: bool,
    dmc_timer: u16,
    dmc_out: u8,
    pub dmc_irq: bool,
    pub frame_irq: bool,
    frame_mode: u8,
    frame_cycle: u32,
    reset_delay: u8,
    cycle: u64,
    pub audio: Vec<i16>,
    pub drained: Vec<i16>,
    resampler: crate::audio::Resampler,
    #[cfg(feature="trace")]
    debug_last: f64,
    #[cfg(feature="trace")]
    debug_mute: u8,
}
impl Apu {
    pub fn new() -> Self {
        Self {
            pulse: [Pulse::default(), Pulse::default()],
            tri: Triangle::default(),
            noise: Noise::default(),
            dmc_r: [0; 4],
            dmc_addr: 0xc000,
            dmc_left: 0,
            dmc_buffer: None,
            dmc_shift: 0,
            dmc_bits: 8,
            dmc_silent: true,
            dmc_timer: 419,
            dmc_out: 0,
            dmc_irq: false,
            frame_irq: false,
            frame_mode: 0,
            frame_cycle: 8,
            reset_delay: 0,
            cycle: 8,
            audio: Vec::with_capacity(1500),
            drained: Vec::with_capacity(1500),
            resampler: crate::audio::Resampler::new(),

            #[cfg(feature="trace")]
            debug_last: -1.0,

            #[cfg(feature="trace")]
            debug_mute: 0,
        }
    }

    #[cfg(feature="trace")]
    pub fn debug_mute(&mut self, mask: u8) { self.debug_mute = mask }
    #[cfg(feature="trace")]
    pub fn debug_noise(&mut self, seed: u16, timer: u16) {
        self.noise.shift = seed;
        self.noise.timer = timer
    }
    fn quarter(&mut self) {
        for p in &mut self.pulse { p.env.clock(p.r[0]) }
        self.noise.env.clock(self.noise.r[0]);
        self.tri.quarter()
    }
    fn half(&mut self) {
        self.pulse[0].half(true);
        self.pulse[1].half(false);
        if self.tri.r[0] & 128 == 0 && self.tri.length > 0 {
            self.tri.length -= 1
        }
        if self.noise.r[0] & 32 == 0 && self.noise.length > 0 {
            self.noise.length -= 1
        }
    }
    pub fn write(&mut self, a: u16, v: u8) {
        match a {
            0x4000..=0x4007 =>
                self.pulse[((a - 0x4000) / 4) as
                            usize].write((a & 3) as usize, v),
            0x4008..=0x400b => self.tri.write((a & 3) as usize, v),
            0x400c..=0x400f => self.noise.write((a & 3) as usize, v),
            0x4010..=0x4013 => {
                self.dmc_r[(a & 3) as usize] = v;
                if a == 0x4010 && v & 128 == 0 { self.dmc_irq = false }
                if a == 0x4011 { self.dmc_out = v & 127 }
            }
            0x4015 => {
                for i in 0..2 {
                    self.pulse[i].enable = v & (1 << i) != 0;
                    if !self.pulse[i].enable { self.pulse[i].length = 0 }
                }
                self.tri.enable = v & 4 != 0;
                if !self.tri.enable { self.tri.length = 0 }
                self.noise.enable = v & 8 != 0;
                if !self.noise.enable { self.noise.length = 0 }
                if v & 16 == 0 {
                    self.dmc_left = 0
                } else if self.dmc_left == 0 { self.dmc_restart() }
                self.dmc_irq = false
            }
            0x4017 => {
                self.frame_mode = v;
                if v & 64 != 0 { self.frame_irq = false }
                self.reset_delay = if self.cycle & 1 == 0 { 4 } else { 3 }
            }
            _ => {}
        }
    }
    fn dmc_restart(&mut self) {
        self.dmc_addr = 0xc000 + self.dmc_r[2] as u16 * 64;
        self.dmc_left = self.dmc_r[3] as u16 * 16 + 1
    }
    pub fn dmc_need(&self) -> bool {
        self.dmc_buffer.is_none() && self.dmc_left != 0
    }
    pub fn dmc_supply(&mut self, v: u8) {
        self.dmc_buffer = Some(v);
        self.dmc_addr =
            if self.dmc_addr == 65535 { 0x8000 } else { self.dmc_addr + 1 };
        self.dmc_left -= 1;
        if self.dmc_left == 0 {
            if self.dmc_r[0] & 64 != 0 {
                self.dmc_restart()
            } else if self.dmc_r[0] & 128 != 0 { self.dmc_irq = true }
        }
    }
    pub fn status(&mut self, open: u8) -> u8 {
        let v =
            ((self.pulse[0].length > 0) as u8) |
                                        (((self.pulse[1].length > 0) as u8) << 1) |
                                    (((self.tri.length > 0) as u8) << 2) |
                                (((self.noise.length > 0) as u8) << 3) |
                            (((self.dmc_left > 0) as u8) << 4) | (open & 32) |
                    ((self.frame_irq as u8) << 6) | ((self.dmc_irq as u8) << 7);
        self.frame_irq = false;
        v
    }
    pub fn tick(&mut self) {
        self.cycle += 1;
        self.frame_cycle += 1;
        if self.reset_delay > 0 {
            self.reset_delay -= 1;
            if self.reset_delay == 0 {
                self.frame_cycle = 0;
                if self.frame_mode & 128 != 0 { self.quarter(); self.half() }
            }
        }
        match self.frame_cycle {
            7457 | 22371 => self.quarter(),
            14913 => { self.quarter(); self.half() }
            29828..=29830 =>
                if self.frame_mode & 128 == 0 {
                    if self.frame_mode & 64 == 0 { self.frame_irq = true }
                    if self.frame_cycle == 29829 { self.quarter(); self.half() }
                    if self.frame_cycle == 29830 { self.frame_cycle = 0 }
                },
            37281 =>
                if self.frame_mode & 128 != 0 { self.quarter(); self.half() },
            37282 => if self.frame_mode & 128 != 0 { self.frame_cycle = 0 },
            _ => {}
        }
        self.tri.timer();
        if self.cycle & 1 != 0 { self.noise.timer(); }
        if self.cycle & 1 == 0 {
            self.pulse[0].timer();
            self.pulse[1].timer()
        }
        if self.dmc_timer == 0 {
            self.dmc_timer = DMC[(self.dmc_r[0] & 15) as usize] - 1;
            if !self.dmc_silent {
                if self.dmc_shift & 1 != 0 {
                    if self.dmc_out <= 125 { self.dmc_out += 2 }
                } else if self.dmc_out >= 2 { self.dmc_out -= 2 }
            }
            self.dmc_shift >>= 1;
            self.dmc_bits -= 1;
            if self.dmc_bits == 0 {
                self.dmc_bits = 8;
                if let Some(v) = self.dmc_buffer.take() {
                    self.dmc_silent = false;
                    self.dmc_shift = v
                } else { self.dmc_silent = true }
            }
        } else { self.dmc_timer -= 1 }
        #[cfg(not(feature="trace"))]
        let p =
            self.pulse[0].output(true) as f64 +
                self.pulse[1].output(false) as f64;
        #[cfg(not(feature="trace"))]
        let t =
            3.0 * self.tri.output() as f64 + 2.0 * self.noise.output() as f64
                + self.dmc_out as f64;
        #[cfg(feature="trace")]
        let (p, t) =
            {
                let m = self.debug_mute;
                (if m & 1 == 0 {
                            self.pulse[0].output(true) as f64
                        } else { 0.0 } +
                        if m & 2 == 0 {
                            self.pulse[1].output(false) as f64
                        } else { 0.0 },
                    if m & 4 == 0 {
                                3.0 * self.tri.output() as f64
                            } else { 0.0 } +
                            if m & 8 == 0 {
                                2.0 * self.noise.output() as f64
                            } else { 0.0 } +
                        if m & 16 == 0 { self.dmc_out as f64 } else { 0.0 })
            };
        let out =
            if p > 0.0 { 95.52 / (8128.0 / p + 100.0) } else { 0.0 } +
                if t > 0.0 { 163.67 / (24329.0 / t + 100.0) } else { 0.0 };

        #[cfg(feature="trace")]
        if out != self.debug_last {
            self.debug_last = out;
            eprintln!("MIX {} {} {} {} {} {}",self.cycle,self.pulse[0].output(true),self.pulse[1].output(false),self.tri.output(),self.noise.output(),self.dmc_out);
        }
        if let Some(sample) = self.resampler.clock(out) {
            self.audio.push(sample)
        }
    }
    pub fn drain(&mut self) -> usize {
        std::mem::swap(&mut self.drained, &mut self.audio);
        self.audio.clear();
        self.drained.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_frame_counter_writes_reset_to_the_same_irq_phase() {
        for write_cycle in [16, 17] {
            let mut apu = Apu::new();
            while apu.cycle < write_cycle { apu.tick(); }
            apu.write(0x4017, 0);
            while apu.cycle < 29847 { apu.tick(); }
            assert!(!apu.frame_irq);
            // The three-cycle assertion window also reasserts an IRQ cleared
            // by $4015 on either of its first two cycles.
            for _ in 0..3 {
                apu.tick();
                assert_eq!(apu.status(0) & 64, 64);
                assert!(!apu.frame_irq);
            }
            apu.tick();
            assert_eq!(apu.status(0) & 64, 0);
        }
    }
}
