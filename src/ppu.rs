use crate::cart::Cart;#[derive(Clone,Copy,Default)]
struct Sprite {
    x: u8,
    attr: u8,
    lo: u8,
    hi: u8,
    index: u8,
}
pub struct Ppu {
    pub fb: Vec<u16>,
    pub scan: u16,
    pub dot: u16,
    pub frame: u64,
    pub ticks: u64,
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub oam: [u8; 256],
    pub oam_addr: u8,
    nt: [u8; 4096],
    pal: [u8; 32],
    v: u16,
    t: u16,
    x: u8,
    w: bool,
    buf: u8,
    bus: u8,
    bus_time: [u64; 8],
    lo: u16,
    hi: u16,
    atlo: u16,
    athi: u16,
    tile: u8,
    attr: u8,
    patlo: u8,
    pathi: u8,
    sprites: [Sprite; 8],
    next: [Sprite; 8],
    count: usize,
    next_count: usize,
    secondary: [u8; 32],
    indices: [u8; 8],
    eval_n: u8,
    eval_m: u8,
    eval_pos: usize,
    eval_latch: u8,
    eval_done: bool,
    overflow_left: u8,
    pub nmi_line: bool,
    suppress: bool,
    skip_render: bool,
}
impl Ppu {
    pub fn new() -> Self {
        Self {
            fb: vec![0;61440],
            scan: 0,
            dot: 25,
            frame: 0,
            ticks: 25,
            ctrl: 0,
            mask: 0,
            status: 0,
            oam: [0; 256],
            oam_addr: 0,
            nt: [0; 4096],
            pal: [0; 32],
            v: 0,
            t: 0,
            x: 0,
            w: false,
            buf: 0,
            bus: 0,
            bus_time: [0; 8],
            lo: 0,
            hi: 0,
            atlo: 0,
            athi: 0,
            tile: 0,
            attr: 0,
            patlo: 0,
            pathi: 0,
            sprites: [Sprite::default(); 8],
            next: [Sprite::default(); 8],
            count: 0,
            next_count: 0,
            secondary: [255; 32],
            indices: [255; 8],
            eval_n: 0,
            eval_m: 0,
            eval_pos: 0,
            eval_latch: 0,
            eval_done: false,
            overflow_left: 0,
            nmi_line: false,
            suppress: false,
            skip_render: false,
        }
    }
    fn update_nmi(&mut self) {
        self.nmi_line = self.status & 128 != 0 && self.ctrl & 128 != 0;
    }
    fn bus_set(&mut self, v: u8, mask: u8) {
        self.bus = (self.bus & !mask) | (v & mask);
        for i in 0..8 {
            if mask & (1 << i) != 0 { self.bus_time[i] = self.ticks }
        }
    }
    fn bus_get(&mut self) -> u8 {
        for i in 0..8 {
            if self.ticks - self.bus_time[i] > 3_000_000 {
                self.bus &= !(1 << i)
            }
        }
        self.bus
    }
    pub fn rendering(&self) -> bool { self.mask & 24 != 0 }
    fn rendering_line(&self) -> bool {
        self.rendering() && (self.scan < 240 || self.scan == 261)
    }
    fn pi(a: u16) -> usize {
        let mut i = a as usize & 31;
        if i & 19 == 16 { i -= 16 }
        i
    }
    fn read_mem(&mut self, c: &mut Cart, a: u16) -> u8 {
        let a = a & 0x3fff;
        c.address(a, self.ticks);
        match a {
            0..=0x1fff => c.chr_read(a),
            0x2000..=0x3eff => self.nt[c.nt_index(a)],
            _ => self.pal[Self::pi(a)],
        }
    }
    fn write_mem(&mut self, c: &mut Cart, a: u16, val: u8) {
        let a = a & 0x3fff;
        c.address(a, self.ticks);
        match a {
            0..=0x1fff => c.chr_write(a, val),
            0x2000..=0x3eff => self.nt[c.nt_index(a)] = val,
            _ => self.pal[Self::pi(a)] = val & 63,
        }
    }
    fn inc_x(&mut self) {
        if self.v & 31 == 31 {
            self.v &= !31;
            self.v ^= 0x400
        } else { self.v += 1 }
    }
    fn inc_y(&mut self) {
        if self.v & 0x7000 != 0x7000 {
            self.v += 0x1000
        } else {
            self.v &= !0x7000;
            let mut y = (self.v >> 5) & 31;
            if y == 29 {
                y = 0;
                self.v ^= 0x800
            } else if y == 31 { y = 0 } else { y += 1 }
            self.v = (self.v & !0x3e0) | (y << 5)
        }
    }
    fn inc_v(&mut self, c: &mut Cart) {
        if self.rendering_line() {
            self.inc_x();
            self.inc_y()
        } else {
            self.v =
                self.v.wrapping_add(if self.ctrl & 4 != 0 { 32 } else { 1 }) &
                    0x7fff
        }
        c.address(self.v & 0x3fff, self.ticks)
    }
    pub fn read_reg(&mut self, c: &mut Cart, reg: u16) -> u8 {
        let open = self.bus_get();
        match reg & 7 {
            2 => {
                let v = (self.status & 224) | (open & 31);
                if self.scan == 241 && self.dot == 1 { self.suppress = true }
                self.status &= 127;
                self.w = false;
                self.update_nmi();
                self.bus_set(v, 224);
                v
            }
            4 => {
                let v =
                    if self.rendering_line() && self.dot >= 1 && self.dot <= 64
                        {
                        255
                    } else if self.rendering_line() && self.dot <= 256 {
                        self.eval_latch
                    } else {
                        let v = self.oam[self.oam_addr as usize];
                        if self.oam_addr & 3 == 2 { v & 227 } else { v }
                    };
                self.bus_set(v, 255);
                v
            }
            7 => {
                let a = self.v & 0x3fff;
                let data = self.read_mem(c, a);
                let v =
                    if a >= 0x3f00 {
                        let v =
                            (data & if self.mask & 1 != 0 { 48 } else { 63 }) |
                                (open & 192);
                        self.buf = self.read_mem(c, a - 0x1000);
                        self.bus_set(v, 63);
                        v
                    } else {
                        let v = self.buf;
                        self.buf = data;
                        self.bus_set(v, 255);
                        v
                    };
                self.inc_v(c);
                v
            }
            _ => open,
        }
    }
    pub fn write_reg(&mut self, c: &mut Cart, reg: u16, val: u8, cycle: u64) {
        self.bus_set(val, 255);
        let r = reg & 7;
        if cycle < 29658 && [0, 1, 5, 6].contains(&r) { return }
        #[cfg(feature="trace")]
        if r == 1 {
            eprintln!("MASK {} {} {} {} {} {}", cycle, self.frame, self.scan,
                self.dot, self.mask, val);
        }
        #[cfg(feature="trace")]
        if r == 0 || r == 5 || r == 6 {
            eprintln!("PPU_WRITE {} {} {} {} {} {} {:04x} {} {}", cycle,
                self.frame, self.scan, self.dot, r, val, self.v, self.x, self.w);
        }
        match r {
            0 => {
                self.ctrl = val;
                self.t = (self.t & !0xc00) | ((val as u16 & 3) << 10);
                self.update_nmi()
            }
            1 => self.mask = val,
            3 => self.oam_addr = val,
            4 => {
                if self.rendering_line() {
                    self.oam_addr = self.oam_addr.wrapping_add(4)
                } else {
                    self.oam[self.oam_addr as usize] = val;
                    self.oam_addr = self.oam_addr.wrapping_add(1)
                }
            }
            5 => {
                if !self.w {
                    self.x = val & 7;
                    self.t = (self.t & !31) | (val as u16 >> 3)
                } else {
                    self.t =
                        (self.t & !0x73e0) | ((val as u16 & 7) << 12) |
                            ((val as u16 & 248) << 2)
                }
                self.w = !self.w
            }
            6 => {
                if !self.w {
                    self.t = (self.t & 255) | ((val as u16 & 63) << 8)
                } else {
                    self.t = (self.t & 0x7f00) | val as u16;
                    self.v = self.t;
                    c.address(self.v & 0x3fff, self.ticks)
                }
                self.w = !self.w
            }
            7 => { self.write_mem(c, self.v, val); self.inc_v(c) }
            _ => {}
        }
    }
    fn load_shift(&mut self) {
        self.lo = (self.lo & 0xff00) | self.patlo as u16;
        self.hi = (self.hi & 0xff00) | self.pathi as u16;
        self.atlo =
            (self.atlo & 0xff00) | if self.attr & 1 != 0 { 255 } else { 0 };
        self.athi =
            (self.athi & 0xff00) | if self.attr & 2 != 0 { 255 } else { 0 }
    }
    fn evaluate(&mut self) {
        let d = self.dot;
        if d == 1 {
            self.eval_pos = 0;
            self.eval_n = self.oam_addr >> 2;
            self.eval_m = 0;
            self.eval_done = false;
            self.overflow_left = 0;
            self.indices = [255; 8];
        }
        if d <= 64 {
            if d & 1 == 0 { self.secondary[(d / 2 - 1) as usize] = 255 }
            self.eval_latch = 255;
            return
        }
        if d > 256 || self.eval_done { return }
        if d & 1 != 0 {
            self.eval_latch =
                self.oam[(self.eval_n as usize * 4 + self.eval_m as usize) &
                        255];
            return
        }
        let height = if self.ctrl & 32 != 0 { 16 } else { 8 };
        let y = if self.scan == 261 { 255 } else { self.scan as u8 };
        let in_range = y >= self.eval_latch && y - self.eval_latch < height;
        if self.eval_pos < 32 {
            if self.eval_m == 0 {
                if in_range {
                    self.indices[self.eval_pos / 4] = self.eval_n;
                    self.secondary[self.eval_pos] = self.eval_latch;
                    self.eval_pos += 1;
                    self.eval_m = 1
                } else { self.eval_n = self.eval_n.wrapping_add(1) }
            } else {
                self.secondary[self.eval_pos] = self.eval_latch;
                self.eval_pos += 1;
                self.eval_m += 1;
                if self.eval_m == 4 {
                    self.eval_m = 0;
                    self.eval_n = self.eval_n.wrapping_add(1)
                }
            }
        } else {
            if self.overflow_left > 0 {
                self.overflow_left -= 1;
                self.eval_m += 1;
                if self.eval_m == 4 {
                    self.eval_m = 0;
                    self.eval_n = self.eval_n.wrapping_add(1)
                }
            } else if in_range {
                self.status |= 32;
                self.overflow_left = 3;
                self.eval_m += 1;
                if self.eval_m == 4 {
                    self.eval_m = 0;
                    self.eval_n = self.eval_n.wrapping_add(1)
                }
            } else {
                self.eval_m = (self.eval_m + 1) & 3;
                self.eval_n = self.eval_n.wrapping_add(1)
            }
        }
        if self.eval_n >= 64 { self.eval_n = 0; self.eval_done = true }
    }
    fn sprite_address(&self, i: usize) -> u16 {
        let y = self.secondary[i * 4];
        let tile = self.secondary[i * 4 + 1];
        let attr = self.secondary[i * 4 + 2];
        let mut row =
            (if self.scan == 261 {
                        255
                    } else { self.scan as u8 }).wrapping_sub(y);
        let tall = self.ctrl & 32 != 0;
        row &= if tall { 15 } else { 7 };
        if attr & 128 != 0 { row ^= if tall { 15 } else { 7 } }
        if tall {
            ((tile as u16 & 1) << 12) + ((tile as u16 & 254) * 16) +
                    ((row as u16 & 8) << 1) + (row as u16 & 7)
        } else {
            ((self.ctrl as u16 & 8) << 9) + tile as u16 * 16 + row as u16
        }
    }
    fn render_pixel(&mut self) {

        #[cfg(feature="trace")]
        if self.scan == 0 && self.dot <= 16 && self.frame < 6 {
            eprintln!("PIX {} {} {:04x} {:02x} {:02x?}",self.frame,self.dot,self.v,self.mask,&self.pal[..16]);
        }
        let x = (self.dot - 1) as usize;
        let bg_on = self.mask & 8 != 0 && (x >= 8 || self.mask & 2 != 0);
        let sp_on = self.mask & 16 != 0 && (x >= 8 || self.mask & 4 != 0);
        let bit = 0x8000 >> self.x;
        let bg =
            if bg_on {
                ((self.lo & bit != 0) as u8) |
                    (((self.hi & bit != 0) as u8) << 1)
            } else { 0 };
        let bp =
            ((self.atlo & bit != 0) as u8) |
                (((self.athi & bit != 0) as u8) << 1);
        let mut spr = 0;
        let mut attr = 0;
        let mut idx = 255;
        // The reference latches the next line's sprite count for the final
        // pixel, while its shift registers still contain the current line.
        let visible_count = if x == 255 {
            self.count.min(self.eval_pos / 4)
        } else { self.count };
        for s in &self.sprites[..visible_count] {
            if s.x == 0 {
                let p =
                    ((s.lo & 128 != 0) as u8) |
                        (((s.hi & 128 != 0) as u8) << 1);
                if p != 0 { spr = p; attr = s.attr; idx = s.index; break }
            }
        }
        if !sp_on { spr = 0 }
        #[cfg(feature="trace")]
        if x == 255 && spr != 0 {
            eprintln!("EDGE_PIXEL {} {} {} bg={} bp={} bg_color={} spr={} attr={} index={} sprite_color={}",
                self.frame, self.scan, self.dot, bg, bp,
                self.pal[if bg != 0 { bp as usize * 4 + bg as usize } else { 0 }],
                spr, attr, idx, self.pal[16 + (attr as usize & 3) * 4 + spr as usize]);
        }
        if bg != 0 && spr != 0 && idx == 0 && x != 255 { self.status |= 64 }
        let p =
            if spr != 0 && (bg == 0 || attr & 32 == 0) {
                16 + (attr as usize & 3) * 4 + spr as usize
            } else if bg != 0 { bp as usize * 4 + bg as usize } else { 0 };
        let mut color =
            if !self.rendering() && self.v & 0x3f00 == 0x3f00 {
                self.pal[Self::pi(self.v)]
            } else { self.pal[p] };
        if self.mask & 1 != 0 { color &= 48 }
        self.fb[self.scan as usize * 256 + x] =
            color as u16 | ((self.mask as u16 & 224) << 1);
        if self.rendering() {
            for s in &mut self.sprites[..self.count] {
                if s.x > 0 { s.x -= 1 } else { s.lo <<= 1; s.hi <<= 1 }
            }
        }
    }
    /// The frame callback exposes the pixel prepared for the next dot. This
    /// does not advance CPU/PPU time or the sprite-zero/overflow status.
    pub fn publish_next_pixel(&mut self) {
        if self.scan != 0 || !(1..=256).contains(&self.dot) { return }
        let shifts = (self.lo, self.hi, self.atlo, self.athi);
        let sprites = self.sprites;
        let status = self.status;
        if self.rendering() && self.dot >= 2 {
            self.lo <<= 1;
            self.hi <<= 1;
            self.atlo <<= 1;
            self.athi <<= 1;
        }
        self.render_pixel();
        (self.lo, self.hi, self.atlo, self.athi) = shifts;
        self.sprites = sprites;
        self.status = status;
    }

    pub fn tick(&mut self, c: &mut Cart) {
        let d = self.dot;
        if self.scan == 261 && d == 338 {
            self.skip_render = self.rendering()
        }
        let line = self.scan < 240 || self.scan == 261;
        if self.scan == 241 && d == 1 {
            if !self.suppress { self.status |= 128 }
            self.suppress = false;
            self.update_nmi()
        }
        if self.scan == 261 && d == 1 { self.status &= 31; self.update_nmi() }
        if self.rendering() && line {
            if d == 0 && self.scan < 240 {
                c.address(((self.ctrl as u16 & 16) << 8) +
                            self.tile as u16 * 16 + ((self.v >> 12) & 7), self.ticks)
            }
            if (2..=257).contains(&d) || (322..=337).contains(&d) {
                self.lo <<= 1;
                self.hi <<= 1;
                self.atlo <<= 1;
                self.athi <<= 1
            }
            if (1..=256).contains(&d) || (321..=336).contains(&d) {
                match (d - 1) & 7 {
                    0 => {
                        self.load_shift();
                        self.tile = self.read_mem(c, 0x2000 | (self.v & 0xfff))
                    }
                    2 => {
                        let a =
                            0x23c0 | (self.v & 0xc00) | ((self.v >> 4) & 0x38) |
                                ((self.v >> 2) & 7);
                        let v = self.read_mem(c, a);
                        self.attr = (v >> (((self.v >> 4) & 4) | (self.v & 2))) & 3
                    }
                    4 =>
                        self.patlo =
                            self.read_mem(c,
                                ((self.ctrl as u16 & 16) << 8) + self.tile as u16 * 16 +
                                    ((self.v >> 12) & 7)),
                    6 =>
                        self.pathi =
                            self.read_mem(c,
                                ((self.ctrl as u16 & 16) << 8) + self.tile as u16 * 16 +
                                        ((self.v >> 12) & 7) + 8),
                    7 => self.inc_x(),
                    _ => {}
                }
            }
            if d == 256 { self.inc_y() }
            if d == 257 {
                self.load_shift();
                self.v = (self.v & !0x41f) | (self.t & 0x41f);
                self.next_count =
                    if self.scan == 261 { 0 } else { self.eval_pos / 4 };
                self.next = [Sprite::default(); 8]
            }
            if self.scan == 261 && (280..=304).contains(&d) {
                self.v = (self.v & !0x7be0) | (self.t & 0x7be0)
            }
            if (1..=256).contains(&d) && self.scan < 240 { self.evaluate() }
            if ((4..=256).contains(&d) || (324..=336).contains(&d)) &&
                    d & 7 == 4 {
                c.address(((self.ctrl as u16 & 16) << 8) +
                            self.tile as u16 * 16 + ((self.v >> 12) & 7), self.ticks)
            }
            if d == 256 || d == 320 {
                c.address(0x2000 | (self.v & 0xfff), self.ticks)
            }
            if (260..=319).contains(&d) && d & 7 == 4 {
                let i = ((d - 260) / 8) as usize;
                c.address(self.sprite_address(i), self.ticks)
            }
            if (257..=320).contains(&d) {
                self.oam_addr = 0;
                let i = ((d - 257) / 8) as usize;
                let p = (d - 257) & 7;
                match p {
                    0 | 2 => { self.read_mem(c, 0x2000 | (self.v & 0xfff)); }
                    4 => {
                        let a = self.sprite_address(i);
                        let lo = self.read_mem(c, a);
                        self.next[i] =
                            Sprite {
                                x: self.secondary[i * 4 + 3],
                                attr: self.secondary[i * 4 + 2],
                                lo: if self.secondary[i * 4 + 2] & 64 != 0 {
                                    lo.reverse_bits()
                                } else { lo },
                                hi: 0,
                                index: self.indices[i],
                            };
                    }
                    6 => {
                        let a = self.sprite_address(i) + 8;
                        let hi = self.read_mem(c, a);
                        self.next[i].hi =
                            if self.next[i].attr & 64 != 0 {
                                hi.reverse_bits()
                            } else { hi };
                    }
                    _ => {}
                }
            }
            if d == 337 {
                self.load_shift();
                self.read_mem(c, 0x2000 | (self.v & 0xfff));
            }
            if d == 339 { self.read_mem(c, 0x2000 | (self.v & 0xfff)); }
        }
        if self.scan < 240 && (1..=256).contains(&d) { self.render_pixel() }
        self.ticks += 1;
        // At power-on frame zero already has the odd-frame phase. The
        // externally observed first run_frame is a no-op; do not use that
        // call count to choose the hardware's skipped-dot phase.
        if self.scan == 261 && d == 339 && self.frame & 1 == 0 &&
                self.skip_render {
            self.dot = 0;
            self.scan = 0;
            self.frame += 1;
            self.sprites = self.next;
            self.count = self.next_count.min(8);
        } else if d == 340 {
            self.dot = 0;
            self.scan += 1;
            self.sprites = self.next;
            self.count = self.next_count.min(8);
            if self.scan == 262 { self.scan = 0; self.frame += 1; }
        } else { self.dot += 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publishing_a_prepared_pixel_does_not_advance_sprite_zero_or_the_clock() {
        let mut ppu = Ppu::new();
        ppu.dot = 2;
        ppu.mask = 0x1e;
        ppu.lo = 0xffff;
        ppu.pal[17] = 42;
        ppu.count = 1;
        ppu.sprites[0] = Sprite { x: 0, attr: 0, lo: 128, hi: 0, index: 0 };
        let phase = (ppu.frame, ppu.scan, ppu.dot, ppu.ticks);
        ppu.publish_next_pixel();
        assert_eq!(ppu.fb[1], 42);
        assert_eq!((ppu.frame, ppu.scan, ppu.dot, ppu.ticks), phase);
        assert_eq!(ppu.status & 64, 0);
        assert_eq!(ppu.sprites[0].lo, 128);
        assert_eq!(ppu.lo, 0xffff);

        let mut rom = vec![0; 16 + 16384];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 1;
        let mut cart = Cart::load(&rom).unwrap();
        ppu.tick(&mut cart);
        assert_eq!(ppu.fb[1], 42);
        assert_eq!(ppu.status & 64, 64);
        assert_eq!(ppu.sprites[0].lo, 0);
    }
}
