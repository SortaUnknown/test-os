#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

extern crate alloc;

pub mod serial;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod task;
pub mod fs;

use conquer_once::spin::OnceCell;
use bootloader_api::info::FrameBufferInfo;
use bootloader_x86_64_common::logger::LockedLogger;
use alloc::vec::Vec;
use spin::Mutex;

pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();
pub static FEED: Mutex<Vec<char>> = Mutex::new(Vec::new());

pub fn init()
{
    gdt::init();
    interrupts::init_idt();

    unsafe{interrupts::PICS.lock().initialize();}

    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> !
{
    loop{x86_64::instructions::hlt();}
}

pub fn init_logger(buffer: &'static mut [u8], info: FrameBufferInfo)
{
    let logger = LOGGER.get_or_init(move || LockedLogger::new(buffer, info, true, false));
    log::set_logger(logger).expect("Logger already set");
    log::set_max_level(log::LevelFilter::Trace);
}