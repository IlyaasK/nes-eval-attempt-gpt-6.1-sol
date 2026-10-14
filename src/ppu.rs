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
    fetch_addr: u16,
    read_hold_active:bool,
    read_hold_high:bool,
    read_hold:u8,
    data_reads: std::collections::VecDeque<(u64,u16)>,
    data_incs: std::collections::VecDeque<(u64,bool)>,
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
    sprite_fetched: bool,
    secondary: [u8; 32],
    secondary_cpu: [u8; 32],
    indices: [u8; 8],
    eval_n: u8,
    eval_m: u8,
    eval_pos: usize,
    eval_latch: u8,
    eval_value: u8,
    eval_done: bool,
    eval_initialized: bool,
    clear_active: bool,
    late_count: Option<usize>,
    post_eval_valid: bool,
    post_eval_n: u8,
    post_eval_latch: u8,
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
            fetch_addr: 0,
            read_hold_active:false,
            read_hold_high:false,
            read_hold:0,
            data_reads: std::collections::VecDeque::new(),
            data_incs: std::collections::VecDeque::new(),
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
            sprite_fetched: false,
            secondary: [255; 32],
            secondary_cpu: [255; 32],
            indices: [255; 8],
            eval_n: 0,
            eval_m: 0,
            eval_pos: 0,
            eval_latch: 0,
            eval_value: 0,
            eval_done: false,
            eval_initialized: false,
            clear_active: false,
            late_count: None,
            post_eval_valid: false,
            post_eval_n: 0,
            post_eval_latch: 0,
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
        self.fetch_addr = a;
        c.address(a, self.ticks);
        match a {
            0..=0x1fff => c.chr_read(a),
            0x2000..=0x3eff => self.nt[c.nt_index(a)],
            _ => self.pal[Self::pi(a)],
        }
    }
    fn read_fetch(&mut self, c: &mut Cart, a: u16) -> u8 {
        self.fetch_addr = a & 0x3fff;
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
        self.fetch_addr = a;
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
        // CPU reads mask the unused primary attribute bits. Internal
        // evaluation retains the raw byte, including unaligned copies.
        if addr & 3 == 2 { value & 227 } else { value }
    }
    pub fn read_reg(&mut self, c: &mut Cart, reg: u16) -> u8 {
        let open = self.bus_get();
        #[cfg(feature="trace")]
        if matches!(reg & 7,2|4|7) {
            eprintln!("PPU_PORT {} {} {} {:04x} {:02x} buf={:02x} oam={} n={} m={} pos={} latch={} mask={}",
                self.scan,self.dot,reg&7,self.v,open,self.buf,self.oam_addr,
                self.eval_n,self.eval_m,self.eval_pos,self.eval_latch,self.mask);
        }
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
                    if self.rendering_line() && self.dot>=322 {
                        self.secondary_cpu[0]
                    } else
                    if self.rendering_line() && self.dot >= 2 && self.dot <= 64
                        {
                        255
                    } else if self.rendering_line() && self.dot <= 321
                        && !((self.scan == 261 || self.scan == 0) && self.dot == 0) {
                        if self.post_eval_valid && self.scan<240 && (66..=257).contains(&self.dot) {
                            self.post_eval_latch
                        } else {self.eval_latch}
                    } else {
                        self.oam_read(self.oam_addr)
                    };
                self.bus_set(v, 255);
                v
            }
            7 => {
                let a = self.v & 0x3fff;
                if self.rendering_line() {
                    // Cartridge pins see the CPU port address immediately.
                    // Palette RAM supplies the CPU immediately and fills
                    // the buffer from its nametable shadow. Buffered reads
                    // instead sample the rendering bus three dots later.
                    c.address(if a>=0x3f00 {a-0x1000} else {a},self.ticks);
                    self.data_reads.push_back((self.ticks+3,if a>=0x3f00 {a-0x1000} else {0xffff}));
                    // Palette bypass also strobes scroll immediately;
                    // buffered reads delay that strobe by four PPU dots.
                    if a>=0x3f00 {self.inc_x();self.inc_y();}
                    else {self.data_incs.push_back((self.ticks+4,false));}
                    return if a>=0x3f00 {
                        let v=(self.pal[Self::pi(a)] & if self.mask&1!=0 {48} else {63}) | (open&192);
                        self.bus_set(v,63); v
                    } else {
                        let v=self.buf; self.bus_set(v,255); v
                    };
                }
                self.read_hold_active=false;
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
    fn initialize_evaluation(&mut self) {
        self.eval_initialized=true;
        self.eval_pos=0;
        self.eval_n=self.oam_addr>>2;
        self.eval_m=self.oam_addr&3;
        self.eval_done=false;
        self.overflow_left=0;
        self.indices=[255;8];
    }
    fn evaluate(&mut self) {
        let d=self.dot;
        if self.scan<240 && d>=65 && !self.eval_initialized {
            let old_count=self.count;
            self.initialize_evaluation();
            self.late_count=Some(old_count);
        }
        if d <= 64 {
            self.clear_active=true;
            if d & 1 == 0 {
                self.secondary[(d / 2 - 1) as usize] = 255;
                self.secondary_cpu[(d / 2 - 1) as usize] = 255;
            }
            self.eval_latch = 255;
            self.eval_value = 255;
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
        if self.eval_done {
            if self.post_eval_valid {
                // Refresh continues reading primary Y on odd dots and the
                // unused secondary slot on even dots. It does not advance
                // the address exposed to CPU writes or sprite selection.
                if d&1!=0 {self.post_eval_latch=self.oam_read(self.post_eval_n*4);}
                else {
                    self.post_eval_latch=self.secondary_cpu[self.eval_pos&31];
                    self.post_eval_n=(self.post_eval_n+1)&63;
                }
            }
            return;
        }
        if d & 1 != 0 {
            let addr = self.eval_n.wrapping_mul(4) | self.eval_m;
            self.eval_value = self.oam[addr as usize];
            self.eval_latch = self.oam_read(addr);
            return
        }
        let height = if self.ctrl & 32 != 0 { 16 } else { 8 };
        let y = if self.scan == 261 { 255 } else { self.scan as u8 };
        let in_range = y >= self.eval_latch && y - self.eval_latch < height;
        if self.eval_pos < 32 {
            if self.eval_pos&3 == 0 {
                // Y is written before the range test. A rejected sprite
                // leaves its Y in the next otherwise empty secondary slot.
                if self.late_count.is_none() || self.clear_active {
                    self.secondary[self.eval_pos]=self.eval_value;
                    self.secondary_cpu[self.eval_pos]=self.eval_latch;
                }
                if in_range {
                    self.indices[self.eval_pos / 4] = self.eval_n;
                    self.eval_pos += 1;
                    self.eval_m=(self.eval_m+1)&3;
                    if self.eval_m==0 {self.eval_n=self.eval_n.wrapping_add(1);}
                } else {
                    self.eval_n=self.eval_n.wrapping_add(1);
                    self.eval_m=0;
                }
            } else {
                if self.late_count.is_none() || self.clear_active {
                    self.secondary[self.eval_pos] = self.eval_value;
                    self.secondary_cpu[self.eval_pos] = self.eval_latch;
                }
                self.eval_pos += 1;
                self.eval_m += 1;
                if self.eval_m == 4 {
                    self.eval_m = 0;
                    self.eval_n = self.eval_n.wrapping_add(1)
                }
                if self.eval_pos&3==0 {self.eval_m=0;}
            }
        } else {
            if self.overflow_left > 0 {
                self.overflow_left -= 1;
                self.eval_m += 1;
                if self.eval_m == 4 {
                    self.eval_m = 0;
                    self.eval_n = self.eval_n.wrapping_add(1)
                }
                if self.overflow_left == 0 {
                    self.eval_done = true;
                    self.eval_m = 0;
                    self.post_eval_valid = true;
                    self.post_eval_n = self.eval_n & 63;
                    self.post_eval_latch = self.secondary_cpu[0];
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
            self.eval_latch = self.secondary_cpu[0];
        }
        if self.eval_n >= 64 {
            self.eval_n=0;self.eval_done=true;
            self.post_eval_valid=true;self.post_eval_n=0;
            self.post_eval_latch=self.secondary_cpu[self.eval_pos&31];
        }
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
        // Unused slots fetch row zero even though their secondary OAM
        // bytes include a vertical flip. Their pixel data is discarded.
        if i >= self.next_count { row = 0; }
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
            self.count.min(self.late_count.unwrap_or(self.eval_pos / 4))
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
        if d==257 {self.sprite_fetched=false;}
        if d==1 {
            self.eval_initialized=false;self.clear_active=false;self.late_count=None;
            if self.render_history&1!=0 {self.initialize_evaluation();}
            self.post_eval_valid=false;
        }
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
            // The first secondary byte remains on the OAM bus through
            // background prefetch and the following line's dot zero.
            if d >= 321 { self.eval_latch = self.secondary_cpu[0]; }
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
                        self.bg_fetch_low = if self.read_hold_active {self.read_hold} else {a as u8};
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
                        self.bg_fetch_low = if self.read_hold_active {self.read_hold} else {a as u8};
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
                        self.bg_fetch_low = if self.read_hold_active {self.read_hold} else {a as u8};
                        self.fetch_addr = (a&0x3f00)|self.bg_fetch_low as u16;
                        if !self.read_hold_high {self.read_hold_active=false;}
                    }
                    5 => {
                        let a = ((self.ctrl as u16 & 16) << 8) |
                            ((self.tile as u16 * 16) & 0xf00) | self.bg_fetch_low as u16;
                        self.fetch_addr = a;
                        self.patlo = c.chr_read(a);
                    }
                    6 => {
                        let a = ((self.ctrl as u16 & 16) << 8) + self.tile as u16 * 16
                            + ((self.v >> 12) & 7) + 8;
                        self.bg_fetch_low = if self.read_hold_active {self.read_hold} else {a as u8};
                        self.fetch_addr = (a&0x3f00)|self.bg_fetch_low as u16;
                        if self.read_hold_high {self.read_hold_active=false;}
                    }
                    7 => {
                        let a = ((self.ctrl as u16 & 16) << 8) |
                            ((self.tile as u16 * 16) & 0xf00) | self.bg_fetch_low as u16;
                        self.fetch_addr = a;
                        self.pathi = c.chr_read(a);
                    }
                    _ => unreachable!()
                }
            }
            if d == 257 {
                self.load_shift();
                self.next_count =
                    if self.scan == 261 { 0 } else { self.late_count.unwrap_or(self.eval_pos / 4) };
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
                self.eval_latch = self.secondary_cpu[i * 4 + (p as usize).min(3)];
                match p {
                    0 | 2 => { self.read_fetch(c, 0x2000 | (self.v & 0xfff)); }
                    4 => {
                        self.sprite_fetched = true;
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
                        self.sprite_fetched = true;
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
                if self.read_hold_active {self.read_hold=self.v as u8;}
                self.load_shift();
                self.read_mem(c, 0x2000 | (self.v & 0xfff));
            }
            if d == 339 {
                if self.read_hold_active {self.read_hold=self.v as u8;}
                self.read_mem(c, 0x2000 | (self.v & 0xfff));
            }
        }
        // Horizontal carry is enabled by the preceding rendering dot,
        // including a mask disable on the data-fetch clock itself.
        if line && self.render_history & 1 != 0 && d & 7 == 0
            && ((8..=256).contains(&d) || (328..=336).contains(&d)) {
            self.inc_x();
        }
        if line && self.render_history & 2 != 0 {
            if d == 256 { self.inc_y() }
            if d == 257 { self.v = (self.v & !0x41f) | (self.t & 0x41f); }
            if self.scan == 261 && (280..=304).contains(&d) {
                self.v = (self.v & !0x7be0) | (self.t & 0x7be0);
            }
        }
        while self.data_reads.front().is_some_and(|&(at,_)|at<=self.ticks) {
            let (_,read_addr)=self.data_reads.pop_front().unwrap();
            let a=if read_addr!=0xffff {read_addr} else if d==336 {0x2000|(self.v&0xfff)} else {self.fetch_addr&0x3fff};
            self.buf=match a {
                0..=0x1fff => c.chr_read(a),
                0x2000..=0x3eff => self.nt[c.nt_index(a)],
                _ => self.pal[Self::pi(a)],
            };
            // The CPU read holds the memory byte on the multiplexed low
            // address latch until the next background pattern strobe.
            // A read at the attribute-data/low-address overlap releases
            // that latch on the high pattern strobe instead.
            self.read_hold=self.buf;
            self.read_hold_active=read_addr==0xffff;
            self.read_hold_high=((1..=255).contains(&d)||(321..=331).contains(&d)) && matches!(d&7,4|5);

        }
        while self.data_incs.front().is_some_and(|&(at,_)|at<=self.ticks) {
            let (_,palette)=self.data_incs.pop_front().unwrap();
            if self.rendering_line() {
                self.inc_x();
                self.inc_y();
                // A palette read retains its external shadow address;
                // moving the internal scroll counter does not expose CHR.
                if !palette {c.address(self.v&0x3fff,self.ticks);}
            } else { self.inc_v(c); }
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
            if self.sprite_fetched {
                self.sprites = self.next;
                self.count = self.next_count.min(8);
            }
        } else if d == 340 {
            self.dot = 0;
            self.scan += 1;
            if self.sprite_fetched {
                self.sprites = self.next;
                self.count = self.next_count.min(8);
            }
            if self.scan == 262 { self.scan = 0; self.frame += 1; }
        } else { self.dot += 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completed_evaluation_refreshes_primary_y_and_retains_the_last_rejected_y() {
        let mut rom=vec![0;16+16384];
        rom[..4].copy_from_slice(b"NES\x1a");rom[4]=1;
        let mut c=Cart::load(&rom).unwrap();
        let mut p=Ppu::new();
        p.scan=100;p.dot=1;p.mask=10;
        p.oam.fill(255);
        for n in 0..64 {p.oam[n*4]=200+(n%32) as u8;}
        while p.dot<193 {p.tick(&mut c);}
        assert_eq!(p.secondary[0],231);
        assert_eq!(p.eval_pos,0);
        assert_eq!(p.oam_addr,0);
        p.tick(&mut c);
        assert_eq!(p.read_reg(&mut c,4),200);
        p.tick(&mut c);
        assert_eq!(p.read_reg(&mut c,4),231);
        p.tick(&mut c);
        assert_eq!(p.read_reg(&mut c,4),201);
        while p.dot<258 {p.tick(&mut c);}
        assert_eq!(p.read_reg(&mut c,4),231);
        p.write_reg(&mut c,1,0,50000);
        assert_eq!(p.read_reg(&mut c,4),200);
    }

    #[test]
    fn horizontal_increment_enable_uses_the_previous_rendering_dot() {
        let mut rom=vec![0;16+16384];
        rom[..4].copy_from_slice(b"NES\x1a");rom[4]=1;
        let mut c=Cart::load(&rom).unwrap();
        for was_rendering in [false,true] {
            let mut p=Ppu::new();
            p.scan=100;p.dot=8;p.v=0x3f07;
            p.mask=if was_rendering {10} else {0};
            p.render_history=if was_rendering {15} else {0};
            p.write_reg(&mut c,1,if was_rendering {0} else {10},40000);
            p.tick(&mut c);
            assert_eq!(p.v,if was_rendering {0x3f08} else {0x3f07});
        }
    }

    #[test]
    fn skipping_sprite_fetch_does_not_reload_a_depleted_previous_line() {
        let mut rom=vec![0;16+16384];
        rom[..4].copy_from_slice(b"NES\x1a");rom[4]=1;
        let mut c=Cart::load(&rom).unwrap();
        let mut p=Ppu::new();
        p.scan=100;p.dot=250;p.mask=0;p.count=1;p.next_count=1;
        p.sprites[0]=Sprite {x:0,lo:0,hi:0,attr:0,index:0};
        p.next[0]=Sprite {x:100,lo:128,hi:0,attr:0,index:0};
        p.pal[17]=21;
        while p.scan==100 || p.dot<1 {p.tick(&mut c);}
        p.write_reg(&mut c,1,20,40000);
        while p.dot<112 {p.tick(&mut c);}
        assert!(p.fb[101*256..101*256+110].iter().all(|&v|v==0));
    }

    #[test]
    fn rendering_palette_reads_use_the_shadow_buffer_and_immediate_scroll_strobe() {
        let mut rom=vec![0;16+16384+8192];
        rom[..4].copy_from_slice(b"NES\x1a");rom[4]=1;rom[5]=1;
        let mut c=Cart::load(&rom).unwrap();
        let mut p=Ppu::new();
        p.scan=100;p.dot=22;p.mask=10;p.render_history=15;
        p.v=0x3f00;p.buf=0xa5;p.pal[0]=63;
        p.nt[c.nt_index(0x2f00)]=226;
        assert_eq!(p.read_reg(&mut c,7),63);
        assert_eq!(p.v,0x4f01);
        assert_eq!(p.buf,0xa5);
        for _ in 0..4 {p.tick(&mut c);}
        assert_eq!(p.buf,226);
        assert!(!p.read_hold_active);
    }

    #[test]
    fn full_secondary_oam_alternates_buses_then_refreshes_y_after_overflow() {
        let mut rom=vec![0;16+16384];
        rom[..4].copy_from_slice(b"NES\x1a");rom[4]=1;
        let mut c=Cart::load(&rom).unwrap();
        for all_eligible in [false,true] {
            let mut p=Ppu::new();
            p.scan=160;p.dot=0;p.mask=20;
            for n in 0..64 {
                p.oam[n*4..n*4+4].copy_from_slice(&[
                    if all_eligible || n<8 {160} else {200+(n%32) as u8},
                    64+n as u8,128+n as u8,192+n as u8]);
            }
            while p.dot<133 {p.tick(&mut c);}
            assert_eq!(p.read_reg(&mut c,4),160); // Secondary bus after an even dot.
            p.tick(&mut c);
            assert_eq!(p.read_reg(&mut c,4),if all_eligible {128} else {130});
            while p.dot<137 {p.tick(&mut c);}
            if all_eligible {
                assert!(p.eval_done);
                assert_eq!(p.read_reg(&mut c,4),160);
            }
            while p.dot<257 {p.tick(&mut c);}
            assert_eq!(p.status&32,if all_eligible {32} else {0});
        }
    }

    #[test]
    fn rendering_reads_drive_the_mapper_but_palette_reads_keep_the_shadow_bus() {
        for palette in [false,true] {
            let mut rom=vec![0;16+32768+8192];
            rom[..4].copy_from_slice(b"NES\x1a");rom[4]=2;rom[5]=1;rom[6]=0x40;
            let mut c=Cart::load(&rom).unwrap();
            c.write(0xc000,0,0);c.write(0xc001,0,1);c.write(0xe001,0,2);
            c.address(0,0);
            let mut p=Ppu::new();
            p.scan=100;p.dot=252;p.ticks=100;p.mask=10;p.render_history=15;
            p.v=if palette {0x7f00} else {0x1000};
            p.read_reg(&mut c,7);
            assert_eq!(c.irq,!palette);
            for _ in 0..5 {p.tick(&mut c);}
            assert_eq!(c.irq,!palette);
        }
    }

    #[test]
    fn a_rendering_read_delays_the_buffer_and_scroll_and_changes_the_fetch_latch() {
        let mut rom=vec![0;16+16384+8192];
        rom[..4].copy_from_slice(b"NES\x1a");rom[4]=1;rom[5]=1;
        rom[16+16384+0xcc9]=255;
        let mut c=Cart::load(&rom).unwrap();
        let mut p=Ppu::new();
        p.scan=100;p.dot=22;p.mask=10;p.render_history=15;
        p.v=0x4184;p.tile=251;p.bg_fetch_low=0xb4;p.buf=0xa5;
        p.nt[c.nt_index(0x2185)]=201;
        for i in [1,5,9,13] {p.pal[i]=17;}
        assert_eq!(p.read_reg(&mut c,7),0xa5);
        assert_eq!(p.buf,0xa5);
        for _ in 0..4 {p.tick(&mut c);}
        assert_eq!(p.buf,201);
        assert_eq!(p.v,0x4185);
        p.tick(&mut c);
        assert_eq!(p.v,0x5186);
        while p.dot<50 {p.tick(&mut c);}
        assert_eq!(&p.fb[100*256+40..100*256+48],&[17;8]);
    }

    #[test]
    fn empty_sprite_reads_use_row_zero_in_both_sprite_sizes() {
        for tall in [false,true] {
            let mut rom=vec![0;16+16384+8192];
            rom[..4].copy_from_slice(b"NES\x1a");rom[4]=1;rom[5]=1;
            let base=if tall {0x1fe0} else {0xff0};
            rom[16+16384+base]=0x31;
            rom[16+16384+base+8]=0x52;
            let mut c=Cart::load(&rom).unwrap();
            let mut p=Ppu::new();
            p.scan=100;p.dot=260;p.mask=10;p.render_history=15;
            p.ctrl=if tall {32} else {0};p.secondary.fill(255);
            assert_eq!(p.read_reg(&mut c,7),0);
            for _ in 0..4 {p.tick(&mut c);}
            assert_eq!(p.buf,0x52);
        }
    }

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
    fn unaligned_oam_selection_copies_four_raw_bytes_and_preserves_the_cpu_bus() {
        let mut rom = vec![0; 16 + 16384 + 8192];
        rom[..4].copy_from_slice(b"NES\x1a");
        rom[4] = 1;
        rom[5] = 1;
        rom[16+16384+170*16..16+16384+170*16+8].fill(255);
        let mut cart = Cart::load(&rom).unwrap();
        let mut ppu = Ppu::new();
        ppu.scan = 160;
        ppu.dot = 0;
        ppu.mask = 20;
        ppu.oam.fill(255);
        ppu.oam[..8].copy_from_slice(&[200,160,170,16,161,200,200,160]);
        ppu.oam_addr = 1;
        while ppu.scan == 160 { ppu.tick(&mut cart); }
        assert_eq!(&ppu.secondary[..4], &[160,170,16,161]);
        assert_eq!(&ppu.secondary_cpu[..4], &[160,162,16,161]);
        ppu.pal[17] = 21;
        while ppu.scan == 161 { ppu.tick(&mut cart); }
        assert!(ppu.fb[161*256+161..161*256+169].iter().all(|&v|v==21));
        assert_eq!(ppu.fb[161*256+160],0);
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
        assert_eq!(ppu.secondary[2],255); // Evaluation copies the raw attribute byte.
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
        assert_eq!(&ppu.secondary[..4], &[232,0,255,40]);
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
