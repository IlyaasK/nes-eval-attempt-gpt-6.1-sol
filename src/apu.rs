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
    pending_length: Option<u8>,
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
        // A register reload reaches the length counter after this clock's
        // sequencer gate is sampled, before the frame counter clocks it.
        let old_length = self.length;
        if let Some(length) = self.pending_length.take() { self.length = length; }
        // Ultrasonic periods hold the DAC and the remaining divider count.
        // A later audible period resumes that count, rather than starting
        // at whichever phase a silently running tiny divider would reach.
        if self.period <= 1 { return }
        if self.timer == 0 {
            self.timer = self.period;
            if old_length > 0 && self.linear > 0 {
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
                if self.enable { self.pending_length = Some(LEN[(v >> 3) as usize]); }
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
    pub dmc_abort_at: Option<u64>,
    pub dmc_extra_at: Option<u64>,
    dmc_buffer_clock: Option<u64>,
    pub dmc_output_clock: Option<u64>,
    dmc_buffer: Option<u8>,
    dmc_shift: u8,
    dmc_bits: u8,
    dmc_silent: bool,
    dmc_timer: u16,
    dmc_out: u8,
    pub dmc_irq: bool,
    pub frame_irq: bool,
    frame_mode: u8,
    frame_active_five: bool,
    frame_cycle: u32,
    last_quarter_clock: Option<u64>,
    last_half_clock: Option<u64>,
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
            dmc_abort_at: None,
            dmc_extra_at: None,
            dmc_buffer_clock: None,
            dmc_output_clock: None,
            dmc_buffer: None,
            dmc_shift: 0,
            dmc_bits: 8,
            dmc_silent: true,
            dmc_timer: 422,
            dmc_out: 0,
            dmc_irq: false,
            frame_irq: false,
            frame_mode: 0,
            frame_active_five: false,
            frame_cycle: 8,
            last_quarter_clock: None,
            last_half_clock: None,
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
        // A natural clock and the five-step reset can overlap. Its quarter
        // and half pulses have independent two-clock windows; an extra
        // pulse one CPU clock later does not clock that unit twice.
        if self.last_quarter_clock.is_some_and(|t| self.cycle - t < 2) { return; }
        self.last_quarter_clock = Some(self.cycle);
        for p in &mut self.pulse { p.env.clock(p.r[0]) }
        self.noise.env.clock(self.noise.r[0]);
        self.tri.quarter()
    }
    fn half(&mut self) {
        if self.last_half_clock.is_some_and(|t| self.cycle - t < 2) { return; }
        self.last_half_clock = Some(self.cycle);
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
        #[cfg(feature="trace")]
        if a == 0x4010 || a == 0x4015 {
            eprintln!("DMC_STATE {} {:04x} {} rate={} timer={} bits={} silent={} shift={} buffer={:?} left={}",
                self.cycle,a,v,self.dmc_r[0],self.dmc_timer,self.dmc_bits,
                self.dmc_silent,self.dmc_shift,self.dmc_buffer,self.dmc_left);
        }
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
                if !self.tri.enable { self.tri.length = 0; self.tri.pending_length = None; }
                self.noise.enable = v & 8 != 0;
                if !self.noise.enable { self.noise.length = 0 }
                if v & 16 == 0 {
                    if self.dmc_left != 0 {
                        // Stopping in the APU cycle before a buffered byte
                        // transfers schedules a one-clock aborted reload.
                        // The following APU cycle retains a full request,
                        // even though its reader length is now zero.
                        if self.dmc_buffer.is_some() && self.dmc_bits == 1 {
                            match self.dmc_timer {
                                1 | 2 => self.dmc_abort_at =
                                    Some(self.cycle + self.dmc_timer as u64 + 1),
                                0 => self.dmc_extra_at = Some(self.cycle + 1),
                                _ => {},
                            }
                        } else if self.dmc_buffer_clock == Some(self.cycle) {
                            self.dmc_extra_at = Some(self.cycle);
                        }
                    }
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
        // A retained request after a stop still fills the output buffer.
        // It does not restart the disabled reader or underflow its length.
        if self.dmc_left == 0 { return; }
        self.dmc_left -= 1;
        if self.dmc_left == 0 {
            if self.dmc_r[0] & 64 != 0 {
                self.dmc_restart()
            } else {
                // A one-byte load can disable its reader in the APU cycle
                // before that byte enters the shifter. The reference keeps
                // the resulting single halt, but not the later-chip extra
                // full reload when this implicit stop is one clock later.
                if self.dmc_bits == 1 && matches!(self.dmc_timer, 1 | 2) {
                    self.dmc_abort_at =
                        Some(self.cycle + self.dmc_timer as u64 + 1);
                }
                if self.dmc_r[0] & 128 != 0 { self.dmc_irq = true }
            }
        }
        // A continuing reader can deliver the first byte one clock after
        // an empty output boundary. The reference accepts that byte into
        // the new shifter packet and immediately prefetches the next one.
        // An implicitly ended one-byte reader does not perform this handoff.
        if self.dmc_left != 0 && self.dmc_silent &&
            self.dmc_output_clock == Some(self.cycle - 1) {
            self.dmc_buffer = None;
            self.dmc_shift = v;
            self.dmc_silent = false;
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
        // A timer edge sees the old linear/length counters. A coincident
        // frame-counter edge changes the gate for the following timer edge.
        self.tri.timer();
        self.frame_cycle += 1;
        if self.reset_delay > 0 {
            self.reset_delay -= 1;
            if self.reset_delay == 0 {
                self.frame_cycle = 0;
                self.frame_active_five = self.frame_mode & 128 != 0;
                if self.frame_active_five { self.quarter(); self.half() }
            }
        }
        match self.frame_cycle {
            7457 | 22371 => self.quarter(),
            14913 => { self.quarter(); self.half() }
            29828..=29830 =>
                if !self.frame_active_five {
                    if self.frame_mode & 64 == 0 { self.frame_irq = true }
                    if self.frame_cycle == 29829 { self.quarter(); self.half() }
                    if self.frame_cycle == 29830 { self.frame_cycle = 0 }
                },
            37281 =>
                if self.frame_active_five { self.quarter(); self.half() },
            37282 => if self.frame_active_five { self.frame_cycle = 0 },
            _ => {}
        }
        // The fixed power-on alignment places the half-rate timer edge on
        // odd numbered CPU cycles, independently of frame-counter resets.
        if self.cycle & 1 != 0 {
            self.noise.timer();
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
                self.dmc_output_clock = Some(self.cycle);
                if let Some(v) = self.dmc_buffer.take() {
                    self.dmc_buffer_clock = Some(self.cycle);
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
    fn ultrasonic_triangle_periods_hold_the_dac_without_disabling_counters() {
        for period in [0, 1] {
            let mut apu = Apu::new();
            apu.write(0x4015, 4);
            apu.write(0x4008, 255);
            apu.write(0x400a, period);
            apu.write(0x400b, 24);
            apu.write(0x4017, 192);
            for _ in 0..1000 { apu.tick(); }
            assert_eq!(apu.tri.output(), 15);
            assert_eq!(apu.tri.seq, 0);
            assert_eq!(apu.status(0) & 4, 4);
            apu.write(0x400a, 2);
            for _ in 0..30 { apu.tick(); }
            assert_ne!(apu.tri.seq, 0);
        }
    }

    #[test]
    fn one_clock_late_dmc_delivery_requires_a_continuing_reader() {
        for (length,looping,expected) in [(0,false,64),(1,false,66),(0,true,66)] {
            let mut apu = Apu::new();
            while apu.cycle < 22 { apu.tick(); }
            apu.write(0x4010,15 | if looping {64} else {0});
            apu.write(0x4011,64);
            apu.write(0x4013,length);
            while apu.cycle < 804 { apu.tick(); }
            apu.write(0x4015,16);
            while apu.cycle < 810 { apu.tick(); }
            apu.dmc_supply(255);
            while apu.cycle < 863 { apu.tick(); }
            assert_eq!(apu.dmc_out,expected);
        }
    }
    #[test]
    fn overlapping_frame_quarter_and_half_pulses_are_suppressed_independently() {
        for (edge, gap, decay, length) in [
            (7479,1,15,1), (7479,3,14,1),
            (14935,1,14,1), (14935,3,13,0),
        ] {
            let mut apu = Apu::new();
            apu.pulse[0].enable = true;
            apu.pulse[0].length = 2;
            apu.noise.enable = true;
            apu.noise.length = 10;
            apu.noise.r[0] = 32;
            apu.noise.env.start = true;
            while apu.cycle < 18 { apu.tick(); }
            apu.write(0x4017,64);
            let write = edge + gap - 4;
            while apu.cycle < write { apu.tick(); }
            apu.write(0x4017,192);
            while apu.cycle < edge+gap { apu.tick(); }
            assert_eq!(apu.noise.env.decay,decay);
            assert_eq!(apu.pulse[0].length,length);
        }
    }
    #[test]
    fn triangle_length_reload_reaches_the_gate_after_a_coincident_timer_edge() {
        // Original seeded streams restore a stopped triangle exactly one
        //clock before its divider expires. The reference skips that edge,
        //loads the new divider, then advances after the whole new period.
        let mut tri = Triangle::default();
        tri.enable = true;
        tri.linear = 16;
        tri.seq = 24;
        tri.period = 2;
        tri.timer = 0;
        tri.write(3,215);
        assert_eq!(tri.period,1794);
        tri.timer();
        assert_eq!(tri.output(),8);
        assert!(tri.length>0);
        for _ in 0..1794 { tri.timer(); }
        assert_eq!(tri.output(),8);
        tri.timer();
        assert_eq!(tri.output(),9);
    }
    #[test]
    fn ultrasonic_triangle_resumes_the_remaining_audible_divider() {
        // Isolated original register stream: lowering period84 to1 with
        //62 clocks remaining, then restoring257 after727 clocks, produces
        //the next DAC step63 clocks after restoration in the reference.
        let mut tri = Triangle::default();
        tri.enable = true;
        tri.length = 10;
        tri.linear = 1;
        tri.period = 84;
        tri.timer = 62;
        tri.seq = 21;
        tri.write(2,1);
        for _ in 0..727 { tri.timer(); }
        assert_eq!(tri.timer,62);
        assert_eq!(tri.output(),5);
        tri.write(3,97);
        assert_eq!(tri.period,257);
        for _ in 0..62 { tri.timer(); }
        assert_eq!(tri.output(),5);
        tri.timer();
        assert_eq!(tri.output(),6);
    }
    #[test]
    fn triangle_gate_changes_after_a_coincident_timer_edge() {
        // Original cycle-accounted ROMs align an 85-cycle triangle divider
        // with the first or second quarter-frame edge after a $4017 write.
        // The reference holds DAC 15 in the first case and DAC 8 in the
        // second; clocking the gate first instead holds DAC 14 or DAC 7.
        for (period_write, stop, edge, expected_seq) in
            [(168, false, 7479, 0), (144, true, 14935, 24)]
        {
            let mut apu = Apu::new();
            while apu.cycle < edge {
                apu.tick();
                match apu.cycle {
                    18 => apu.write(0x4017, 64),
                    50 => apu.write(0x4015, 4),
                    60 => apu.write(0x4008, 131),
                    200 => apu.write(0x400b, 0),
                    14835 if stop => apu.write(0x4008, 128),
                    _ => {}
                }
                if apu.cycle == period_write { apu.write(0x400a, 84); }
            }
            assert_eq!(apu.tri.seq, expected_seq);
            assert_eq!(apu.tri.linear, if stop { 0 } else { 3 });
            assert_eq!(apu.tri.output(), if stop { 8 } else { 15 });
        }
    }

    #[test]
    fn power_dmc_phase_matches_reader_and_dac_reference_timestamps() {
        let mut apu = Apu::new();
        apu.write(0x4010, 0x4f);
        apu.write(0x4012, 0x20);
        apu.write(0x4013, 1);
        apu.write(0x4015, 16);
        apu.dmc_supply(255);
        while apu.cycle < 808 { apu.tick(); }
        assert!(!apu.dmc_need());
        apu.tick();
        assert!(apu.dmc_need());
        while apu.cycle < 862 { apu.tick(); }
        assert_eq!(apu.dmc_out, 0);
        apu.tick();
        assert_eq!(apu.dmc_out, 2);
        for _ in 0..54 { apu.tick(); }
        assert_eq!(apu.dmc_out, 4);
    }

    #[test]
    fn frame_mode_is_delayed_but_irq_inhibit_changes_on_the_write() {
        // The original frame_mode_probe reads $4015 four cycles after these
        // writes. Mode $80 keeps the old sequence alive through its IRQ edge;
        // mode $c0 suppresses that same edge immediately.
        for (write_cycle, mode, expected_irq) in [
            (29846, 128, false), (29848, 128, true),
            (29850, 128, true), (29848, 192, false),
        ] {
            let mut apu = Apu::new();
            while apu.cycle < 18 { apu.tick(); }
            apu.write(0x4017, 64);
            while apu.cycle < write_cycle { apu.tick(); }
            apu.write(0x4017, mode);
            for _ in 0..4 { apu.tick(); }
            assert_eq!(apu.status(0) & 64 != 0, expected_irq);
        }
    }

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
