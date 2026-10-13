pub mod cart;
pub mod ppu;
pub mod apu;
pub mod audio;
pub mod cpu;
use cpu::Nes;
use std::cell::UnsafeCell;
struct State{rom:Vec<u8>,nes:Option<Nes>,saved:Vec<u8>}
struct Global(UnsafeCell<Option<State>>);
unsafe impl Sync for Global{}
static STATE:Global=Global(UnsafeCell::new(None));
unsafe fn state()->&'static mut State {let s=&mut *STATE.0.get();if s.is_none(){*s=Some(State{rom:vec![0;8*1024*1024],nes:None,saved:Vec::new()})}s.as_mut().unwrap()}
#[no_mangle] pub extern "C" fn emu_init()->i32{unsafe{state();}1}
#[no_mangle] pub extern "C" fn emu_rom_buffer()->*mut u8{unsafe{state().rom.as_mut_ptr()}}
#[no_mangle] pub extern "C" fn emu_load_rom(len:i32)->i32{unsafe{let s=state();if len<0||len as usize>s.rom.len(){return 0}if let Some(c)=cart::Cart::load(&s.rom[..len as usize]){s.saved=s.rom[..len as usize].to_vec();s.nes=Some(Nes::new(c));1}else{0}}}
#[no_mangle] pub extern "C" fn emu_reset()->i32{unsafe{let s=state();if let Some(c)=cart::Cart::load(&s.saved){s.nes=Some(Nes::new(c));1}else{0}}}
#[no_mangle] pub extern "C" fn emu_set_keys(k:u32){unsafe{if let Some(n)=state().nes.as_mut(){n.set_keys(k as u8)}}}
#[no_mangle] pub extern "C" fn emu_run_frame(){unsafe{if let Some(n)=state().nes.as_mut(){n.run_frame()}}}
#[no_mangle] pub extern "C" fn emu_framebuffer()->*mut u16{unsafe{state().nes.as_mut().map_or(std::ptr::null_mut(),|n|n.bus.borrow_mut().ppu.fb.as_mut_ptr())}}
#[no_mangle] pub extern "C" fn emu_audio_buffer()->*mut i16{unsafe{state().nes.as_mut().map_or(std::ptr::null_mut(),|n|n.bus.borrow_mut().apu.drained.as_mut_ptr())}}
#[no_mangle] pub extern "C" fn emu_audio_samples()->i32{unsafe{state().nes.as_mut().map_or(0,|n|n.bus.borrow_mut().apu.drain() as i32)}}
#[no_mangle] pub extern "C" fn emu_audio_rate()->i32{44100}
