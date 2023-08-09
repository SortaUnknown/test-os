#![no_std]
#![cfg_attr(test, no_main)]
/*#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]*/
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

extern crate alloc;

//pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod task;
//pub mod framebuffer;
//pub mod logger;

//use core::panic::PanicInfo;
use conquer_once::spin::OnceCell;
//use x86_64::instructions::port::Port;
use bootloader_api::info::FrameBufferInfo;
use bootloader_x86_64_common::logger::LockedLogger;

pub static LOGGER: OnceCell<LockedLogger> = OnceCell::uninit();

/*#[cfg(test)]
use bootloader_api::{BootInfo, entry_point};

#[cfg(test)]
entry_point!(kernel_start);*/

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

/*pub trait Testable
{
    fn run(&self) -> ();
}

impl<T> Testable for T where T: Fn(),
{
    fn run(&self)
    {
        serial_print!("{}... ", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable])
{
    serial_println!("Running {} tests...", tests.len());
    for test in tests
    {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> !
{
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);

    hlt_loop();
}

//Entry point for "cargo test"
#[cfg(test)]
fn kernel_start(_boot_info: &'static mut BootInfo) -> !
{
    test_main();

    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    test_panic_handler(info)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode
{
    Success = 0x10,
    Failed = 0x11
}

pub fn exit_qemu(exit_code: QemuExitCode)
{
    unsafe
    {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}*/