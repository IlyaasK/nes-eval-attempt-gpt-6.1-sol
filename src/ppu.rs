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
    pending_v: Option<(u16,u64)>,
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
    // Vertical scroll strobes see the rendering enable two dots later.
    render_history: u8,
    // A new background tile becomes transferable after its low fetch.
    bg_ready: bool,
    render_just_enabled: bool,
    bg_fetch_low: u8,
    patlo: u8,
    pathi: u8,
    sprites: [Sprite; 8],
    next: [Sprite; 8],
    count: usize,
    next_count: usize,
    sprite_fetch_addr: [u16;8],
    sprite_fetch_valid: [bool;8],
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
    cart_addr_pending: Option<u16>,
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
            pending_v: None,
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
            render_history: 0,
            bg_ready: false,
            render_just_enabled: false,
            bg_fetch_low: 0,
            patlo: 0,
            pathi: 0,
            sprites: [Sprite::default(); 8],
            next: [Sprite::default(); 8],
            count: 0,
            next_count: 0,
            sprite_fetch_addr: [0;8],
            sprite_fetch_valid: [false;8],
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
            cart_addr_pending: None,
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
    fn read_fetch(&mut self, c: &mut Cart, a: u16) -> u8 {
        if c.mapper != 1 { return self.read_mem(c, a); }
        let a = a & 0x3fff;
        // MMC1 expansion outputs follow the completed rendering fetch one
        // dot later. CPU register accesses and end-of-line garbage fetches
        // use read_mem and update the pins immediately.
        self.cart_addr_pending = Some(a);
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
    fn oam_read(&self, addr: u8) -> u8 {
        let value = self.oam[addr as usize];
        // Primary OAM's unused attribute bits read as zero, including
        // evaluation. Clearing secondary OAM can still put $FF on its bus.
        if addr & 3 == 2 { value & 227 } else { value }
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
                    if self.rendering_line() && self.dot >= 2 && self.dot <= 64
                        {
                        255
                    } else if self.rendering_line() && self.dot <= 321
                        && !((self.scan == 261 || self.scan == 0) && self.dot == 0) {
                        self.eval_latch
                    } else {
                        self.oam_read(self.oam_addr)
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
            eprintln!("OAM_MASK {} {} {} {} {} {} {}", cycle, self.frame, self.scan, self.dot, self.oam_addr, self.eval_n, self.eval_m);
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
            1 => { self.render_just_enabled = !self.rendering() && val & 24 != 0; self.mask = val },
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
                    if self.rendering_line() {
                        self.pending_v = Some((self.t,self.ticks+3));
                    } else { self.v = self.t; self.pending_v = None; }
                    c.address(self.t & 0x3fff, self.ticks)
                }
                self.w = !self.w
            }
            7 => { self.write_mem(c, self.v, val); self.inc_v(c) }
            _ => {}
        }
    }
    fn load_shift(&mut self) {
        if !self.bg_ready { return }
        self.bg_ready = false;
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
        if d > 256 { return }
        if self.scan == 261 {
            // Pre-render walks primary Y bytes without selecting sprites.
            // CPU writes do not redirect the walk. After it wraps, the
            // primary bus follows OAMADDR again until sprite fetch starts.
            if d & 1 != 0 {
                self.eval_latch = self.oam_read(if self.eval_done { self.oam_addr }
                    else { self.eval_n.wrapping_mul(4) });
            } else if !self.eval_done {
                self.eval_n += 1;
                if self.eval_n >= 64 { self.eval_n = 0; self.eval_done = true; }
                if !self.eval_done { self.oam_addr = self.eval_n.wrapping_mul(4); }
            }
            return
        }
        if self.eval_done { return }
        if d & 1 != 0 {
            self.eval_latch = self.oam_read(self.eval_n.wrapping_mul(4) | self.eval_m);
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
        // The evaluator and CPU/DMA share OAMADDR. Forced blanking before
        // evaluation finishes preserves this byte address, so a subsequent
        // DMA can rotate the sprite list if software does not write $2003.
        self.oam_addr = self.eval_n.wrapping_mul(4) | self.eval_m;
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
        if self.scan >= 240 || !(1..=256).contains(&self.dot) { return }
        let shifts = (self.lo, self.hi, self.atlo, self.athi);
        let sprites = self.sprites;
        let status = self.status;
        if self.rendering() && self.dot >= 2 {
            self.lo <<= 1;
            self.hi = (self.hi << 1) | 1;
            self.atlo <<= 1;
            self.athi <<= 1;
        }
        self.render_pixel();
        (self.lo, self.hi, self.atlo, self.athi) = shifts;
        self.sprites = sprites;
        self.status = status;
    }

    pub fn tick(&mut self, c: &mut Cart) {
        if let Some((v,at)) = self.pending_v {
            if self.ticks >= at { self.v = v; self.pending_v = None; }
        }
        if let Some(a) = self.cart_addr_pending.take() { c.address(a, self.ticks); }
        let d = self.dot;
        let just_enabled = std::mem::take(&mut self.render_just_enabled);
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
            if d == 0 { self.eval_latch = self.oam_read(self.oam_addr); }
            // An upper-table background keeps A12 low from the final NT
            // fetch through the next line's first attribute fetch. A dot-0
            // high pulse would erase the qualified MMC3 edge on dot 6.
            if d == 0 && self.scan < 240 && self.ctrl & 16 == 0 {
                c.address(((self.ctrl as u16 & 16) << 8) +
                            self.tile as u16 * 16 + ((self.v >> 12) & 7), self.ticks)
            }
            if (2..=257).contains(&d) || (322..=337).contains(&d) {
                self.lo <<= 1;
                // The upper bitplane shifts in logical ones. Its fill
                // input responds one dot after rendering is enabled.
                self.hi = (self.hi << 1) | !just_enabled as u16;
                self.atlo <<= 1;
                self.athi <<= 1
            }
            if (1..=256).contains(&d) || (321..=336).contains(&d) {
                match (d - 1) & 7 {
                    0 => {
                        self.load_shift();
                        let a = 0x2000 | (self.v & 0xfff);
                        self.bg_fetch_low = a as u8;
                        self.read_fetch(c,a);
                    }
                    1 => {
                        // The external latch holds the previous dot's low
                        // byte; the upper address pins follow live v.
                        let a = 0x2000 | (self.v & 0xf00) | self.bg_fetch_low as u16;
                        self.tile = self.nt[c.nt_index(a)];
                    }
                    2 => {
                        let a = 0x23c0 | (self.v & 0xc00) | ((self.v >> 4) & 0x38) |
                                ((self.v >> 2) & 7);
                        self.bg_fetch_low = a as u8;
                        self.read_fetch(c,a);
                    }
                    3 => {
                        let a = 0x2300 | (self.v & 0xc00) | self.bg_fetch_low as u16;
                        let v = self.nt[c.nt_index(a)];
                        self.attr = (v >> (((self.v >> 4) & 4) | (self.v & 2))) & 3;
                    }
                    4 => {
                        self.bg_ready = true;
                        let a = ((self.ctrl as u16 & 16) << 8) + self.tile as u16 * 16
                            + ((self.v >> 12) & 7);
                        self.bg_fetch_low = a as u8;
                    }
                    5 => {
                        let a = ((self.ctrl as u16 & 16) << 8) |
                            ((self.tile as u16 * 16) & 0xf00) | self.bg_fetch_low as u16;
                        self.patlo = c.chr_read(a);
                    }
                    6 => {
                        let a = ((self.ctrl as u16 & 16) << 8) + self.tile as u16 * 16
                            + ((self.v >> 12) & 7) + 8;
                        self.bg_fetch_low = a as u8;
                    }
                    7 => {
                        let a = ((self.ctrl as u16 & 16) << 8) |
                            ((self.tile as u16 * 16) & 0xf00) | self.bg_fetch_low as u16;
                        self.pathi = c.chr_read(a);
                        self.inc_x();
                    }
                    _ => unreachable!()
                }
            }
            if d == 257 {
                self.load_shift();
                self.next_count =
                    if self.scan == 261 { 0 } else { self.eval_pos / 4 };
                self.next = [Sprite::default(); 8]
            }
            if (1..=256).contains(&d) { self.evaluate() }
            if ((6..=256).contains(&d) || (326..=336).contains(&d))
                && (d & 7 == 6 || d & 7 == 0)
            {
                // Background data is fetched on dots 5/7; its address
                // reaches the cartridge one dot later. Sprite presentation
                // has its separately measured phase at dot 260.
                c.address(((self.ctrl as u16 & 16) << 8) + self.tile as u16 * 16
                    + ((self.v >> 12) & 7), self.ticks);
            }
            if c.mapper == 4 && (d == 256 || d == 320) {
                c.address(0x2000 | (self.v & 0xfff), self.ticks)
            }
            if (260..=319).contains(&d) && d & 7 == 4 {
                let i = ((d - 260) / 8) as usize;
                let a = self.sprite_address(i);
                let y = if self.scan == 261 {255} else {self.scan as u8};
                let row = y.wrapping_sub(self.secondary[i*4]);
                self.sprite_fetch_addr[i] = a;
                self.sprite_fetch_valid[i] = row < if self.ctrl & 32 != 0 {16} else {8};
                c.address(a, self.ticks)
            }
            if (257..=320).contains(&d) {
                self.oam_addr = 0;
                let i = ((d - 257) / 8) as usize;
                let p = (d - 257) & 7;
                // Secondary OAM drives $2004 during sprite fetch. Each
                // slot presents Y/tile/attribute/X on its first four dots,
                // then holds X for the remaining four. CPU reads see the
                // most recently completed dot's value.
                self.eval_latch = self.secondary[i * 4 + (p as usize).min(3)];
                match p {
                    0 | 2 => { self.read_fetch(c, 0x2000 | (self.v & 0xfff)); }
                    4 => {
                        let a = self.sprite_fetch_addr[i];
                        let lo = self.read_fetch(c, a);
                        let lo = if self.sprite_fetch_valid[i] {lo} else {0};
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
                        let a = self.sprite_fetch_addr[i] + 8;
                        let hi = self.read_fetch(c, a);
                        let hi = if self.sprite_fetch_valid[i] {hi} else {0};
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
        if line && self.render_history & 2 != 0 {
            if d == 256 { self.inc_y() }
            if d == 257 { self.v = (self.v & !0x41f) | (self.t & 0x41f); }
            if self.scan == 261 && (280..=304).contains(&d) {
                self.v = (self.v & !0x7be0) | (self.t & 0x7be0);
            }
        }
        if self.scan < 240 && (1..=256).contains(&d) { self.render_pixel() }
        self.render_history = ((self.render_history << 1) | self.rendering() as u8) & 15;
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
    fn sprite_pattern_address_and_range_are_latched_before_both_bitplanes() {
        for tall in [false,true] {
            let mut rom = vec![0;16+16384+8192];
            rom[..4].copy_from_slice(b"NES\x1a"); rom[4]=1; rom[5]=1;
            for a in [1,17,25] { rom[16+16384+a]=255; }
            let mut c = Cart::load(&rom).unwrap();
            let mut p = Ppu::new();
            p.scan=161; p.dot=260; p.mask=30; p.render_history=15;
            p.ctrl=if tall {32} else {0}; p.next_count=1;
            p.secondary[..4].copy_from_slice(&[152,0,0,96]);
            p.pal.fill(1); p.pal[17]=17; p.pal[19]=42;
            p.tick(&mut c);
            p.write_reg(&mut c,0,if tall {0} else {32},77949);
            while p.scan==161 || p.dot<105 { p.tick(&mut c); }
            assert_eq!(&p.fb[162*256+96..162*256+104],
                &[if tall {42} else {1};8]);
        }
    }

    #[test]
    fn enabling_background_after_its_low_fetch_exposes_the_shift_fill_bits() {
        for value in [0,255] {
            let mut rom = vec![0;16+16384+8192];
            rom[..4].copy_from_slice(b"NES\x1a");
            rom[4] = 1; rom[5] = 1;
            rom[16+16384..].fill(value);
            let mut c = Cart::load(&rom).unwrap();
            let mut p = Ppu::new();
            p.scan = 193; p.dot = 198;
            p.pal[0] = 1; p.pal[2] = 3; p.pal[3] = 4;
            p.write_reg(&mut c,1,10,51784);
            while p.dot <= 224 { p.tick(&mut c); }
            assert_eq!(&p.fb[193*256+208..193*256+213], &[1;5]);
            assert_eq!(&p.fb[193*256+213..193*256+216], &[3;3]);
            assert_eq!(&p.fb[193*256+216..193*256+224],
                &[if value==0 {1} else {4};8]);
        }
    }

    #[test]
    fn mid_render_address_transfer_delays_v_and_preserves_the_fetch_low_byte() {
        let mut rom = vec![0;16+16384];
        rom[..4].copy_from_slice(b"NES\x1a"); rom[4] = 1;
        let mut c = Cart::load(&rom).unwrap();
        let mut p = Ppu::new();
        p.scan = 223; p.dot = 121; p.ticks = 1000;
        p.mask = 10; p.render_history = 15; p.v = 0x2000;
        p.nt[c.nt_index(0x2000)] = 5;
        p.nt[c.nt_index(0x23c0)] = 0x81;
        p.write_reg(&mut c,6,0x22,84929);
        p.write_reg(&mut c,6,0x4f,84949);
        assert_eq!(p.v,0x2000);
        p.tick(&mut c); p.tick(&mut c); p.tick(&mut c);
        assert_eq!(p.v,0x2000);
        assert_eq!(p.tile,5);
        p.tick(&mut c);
        assert_eq!(p.v,0x224f);
        assert_eq!(p.attr,2);
    }

    #[test]
    fn enabling_at_the_last_visible_dot_misses_the_vertical_increment() {
        let mut rom = vec![0;16+16384];
        rom[..4].copy_from_slice(b"NES\x1a"); rom[4] = 1;
        let mut c = Cart::load(&rom).unwrap();
        let mut p = Ppu::new();
        p.scan = 159; p.dot = 255; p.v = 0x2000; p.t = 0x2010;
        p.write_reg(&mut c,1,30,77719);
        p.tick(&mut c); p.tick(&mut c); p.tick(&mut c);
        assert_eq!(p.v,0x2010);
        assert_eq!(p.dot,258);
    }

    #[test]
    fn oam_data_follows_primary_clear_and_secondary_fetch_bus_phases() {
        let mut rom = vec![0; 16 + 16384];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 1;
        let mut cart = Cart::load(&rom).unwrap();
        let mut ppu = Ppu::new();
        ppu.scan = 61;
        ppu.dot = 0;
        ppu.mask = 20;
        ppu.oam.fill(255);
        ppu.oam[..8].copy_from_slice(&[60,1,255,40,60,2,1,80]);
        ppu.oam[128..132].copy_from_slice(&[60,3,2,160]);
        ppu.tick(&mut cart);
        assert_eq!(ppu.read_reg(&mut cart,4),60); // Currentdot1, primaryY.
        ppu.tick(&mut cart);
        assert_eq!(ppu.read_reg(&mut cart,4),255); // Clear starts atdot1.
        while ppu.dot < 70 { ppu.tick(&mut cart); }
        assert_eq!(ppu.read_reg(&mut cart,4),227); // Primaryattr's unusedbits.
        while ppu.dot < 257 { ppu.tick(&mut cart); }
        assert_eq!(ppu.secondary[2],227);
        assert_eq!(ppu.read_reg(&mut cart,4),255); // Finalevalbus beforefetch.
        for value in [60,1,227,40,40,40,40,40,60,2,1,80,80,80,80,80] {
            ppu.tick(&mut cart);
            assert_eq!(ppu.read_reg(&mut cart,4),value);
        }
        while ppu.dot < 321 { ppu.tick(&mut cart); }
        assert_eq!(ppu.read_reg(&mut cart,4),255); // LastsecondaryX atdot320.
        ppu.tick(&mut cart);
        assert_eq!(ppu.read_reg(&mut cart,4),60); // PrimaryOAM outsidefetch.
    }

    #[test]
    fn prerender_clears_bottom_line_secondary_sprites_without_evaluating_line_zero() {
        let mut rom = vec![0; 16 + 16384];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 1;
        let mut cart = Cart::load(&rom).unwrap();
        let mut ppu = Ppu::new();
        ppu.scan = 239;
        ppu.dot = 1;
        ppu.mask = 20;
        ppu.oam.fill(255);
        ppu.oam[..4].copy_from_slice(&[232,0,255,40]);
        while ppu.dot < 257 { ppu.tick(&mut cart); }
        assert_eq!(&ppu.secondary[..4], &[232,0,227,40]);
        ppu.scan = 261;
        ppu.dot = 0;
        ppu.oam_addr = 2;
        assert_eq!(ppu.read_reg(&mut cart,4),227); // Vblank's primary bus.
        ppu.oam_addr = 0;
        ppu.tick(&mut cart);
        assert_eq!(ppu.read_reg(&mut cart,4),232);
        while ppu.dot < 65 { ppu.tick(&mut cart); }
        assert!(ppu.secondary.iter().all(|&v|v==255));
        assert_eq!(ppu.read_reg(&mut cart,4),255);
        ppu.oam_addr = 2;
        ppu.tick(&mut cart);
        assert_eq!(ppu.read_reg(&mut cart,4),232); // Internalwalk ignores$2003.
        while ppu.dot < 128 { ppu.tick(&mut cart); }
        assert_eq!(ppu.read_reg(&mut cart,4),255); // Out-of-rangeprimaryY.
        while ppu.dot < 194 { ppu.tick(&mut cart); }
        assert_eq!(ppu.read_reg(&mut cart,4),255);
        assert_eq!(ppu.oam_addr,252); // The final Y address is retained.
        ppu.oam_addr = 0;
        while ppu.dot < 257 { ppu.tick(&mut cart); }
        assert_eq!(ppu.read_reg(&mut cart,4),232);
        ppu.tick(&mut cart);
        assert_eq!(ppu.read_reg(&mut cart,4),255);
        assert_eq!(ppu.next_count,0);
        ppu.scan = 261;
        ppu.dot = 0;
        while ppu.dot < 250 { ppu.tick(&mut cart); }
        ppu.oam_addr = 2;
        ppu.tick(&mut cart);
        ppu.tick(&mut cart);
        assert_eq!(ppu.read_reg(&mut cart,4),227);
        ppu.scan = 0;
        ppu.dot = 0;
        assert_eq!(ppu.read_reg(&mut cart,4),227); // Frameboundaryprimarybus.
    }

    #[test]
    fn mmc3_background_edge_follows_the_data_fetch_and_retains_the_low_interval() {
        let mut rom = vec![0; 16 + 32768 + 8192];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 2;
        rom[5] = 1;
        rom[6] = 0x40;
        let mut cart = Cart::load(&rom).unwrap();
        cart.write(0xc000, 0, 0);
        cart.write(0xc001, 0, 1);
        cart.write(0xe001, 0, 2);
        cart.address(0x1000, 0);
        cart.address(0x2000, 96);
        let mut ppu = Ppu::new();
        ppu.scan = 0;
        ppu.dot = 0;
        ppu.ticks = 100;
        ppu.ctrl = 16;
        ppu.mask = 8;
        while ppu.dot <= 5 { ppu.tick(&mut cart); }
        assert!(!cart.irq);
        ppu.tick(&mut cart);
        assert!(cart.irq);
    }
    #[test]
    fn mmc1_ram_pins_delay_normal_fetches_but_not_end_of_line_fetches() {
        let mut rom = vec![0; 16 + 65536];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 4;
        rom[6] = 0x10;
        rom[7] = 8;
        rom[10] = 8;
        let mut cart = Cart::load(&rom).unwrap();
        let mut cycle = 0;
        for (addr, value) in [(0x8000, 28), (0xa000, 0), (0xc000, 8)] {
            for bit in 0..5 {
                cart.write(addr, (value >> bit) & 1, cycle);
                cycle += 2;
            }
        }
        cart.address(0, 0);
        cart.write(0x6000, 17, cycle);
        cart.address(0x1000, 1);
        cart.write(0x6000, 34, cycle);
        cart.address(0, 2);
        let mut ppu = Ppu::new();
        ppu.scan = 5;
        ppu.dot = 1;
        ppu.ticks = 100;
        ppu.ctrl = 16;
        ppu.mask = 8;
        while ppu.dot <= 6 { ppu.tick(&mut cart); }
        assert_eq!(cart.read(0x6000, 0), 34);
        while ppu.dot <= 9 { ppu.tick(&mut cart); }
        assert_eq!(cart.read(0x6000, 0), 34);
        ppu.tick(&mut cart);
        assert_eq!(cart.read(0x6000, 0), 17);
        while ppu.dot <= 336 { ppu.tick(&mut cart); }
        assert_eq!(cart.read(0x6000, 0), 34);
        ppu.tick(&mut cart);
        assert_eq!(cart.read(0x6000, 0), 17);
    }

    #[test]
    fn forced_blanking_preserves_the_sprite_evaluators_oam_address() {
        let mut ppu = Ppu::new();
        let mut rom = vec![0; 16 + 16384];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 1;
        let mut cart = Cart::load(&rom).unwrap();
        ppu.scan = 61;
        ppu.dot = 1;
        ppu.mask = 0x1e;
        ppu.oam.fill(255);
        ppu.oam[..4].copy_from_slice(&[60, 1, 0, 20]);
        while ppu.dot <= 70 { ppu.tick(&mut cart); }
        assert_eq!(ppu.oam_addr, 3);
        ppu.write_reg(&mut cart, 0x2001, 0, 30000);
        while ppu.scan == 61 { ppu.tick(&mut cart); }
        assert_eq!(ppu.oam_addr, 3);
        ppu.write_reg(&mut cart, 0x2004, 42, 30100);
        assert_eq!(ppu.oam[3], 42);
        assert_eq!(ppu.oam_addr, 4);
    }

    #[test]
    fn final_pixel_uses_the_references_next_line_sprite_count() {
        let mut ppu = Ppu::new();
        ppu.mask = 0x1e;
        ppu.lo = 0xffff;
        ppu.pal[1] = 17;
        ppu.pal[17] = 36;
        ppu.count = 2;
        ppu.eval_pos = 4;
        ppu.sprites[1] = Sprite { x: 0, attr: 0, lo: 128, hi: 0, index: 1 };
        ppu.dot = 255;
        ppu.render_pixel();
        assert_eq!(ppu.fb[254], 36);
        ppu.sprites[1].lo = 128;
        ppu.dot = 256;
        ppu.render_pixel();
        assert_eq!(ppu.fb[255], 17);
        ppu.eval_pos = 8;
        ppu.sprites[1].lo = 128;
        ppu.render_pixel();
        assert_eq!(ppu.fb[255], 36);
    }

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

        // A DMA may finish several visible rows beyond the frame wrap.
        ppu.scan = 2;
        ppu.dot = 233;
        ppu.mask = 0;
        ppu.v = 0;
        ppu.pal[0] = 19;
        let phase = (ppu.frame, ppu.scan, ppu.dot, ppu.ticks);
        ppu.publish_next_pixel();
        assert_eq!(ppu.fb[2*256+232], 19);
        assert_eq!((ppu.frame, ppu.scan, ppu.dot, ppu.ticks), phase);
    }
}
