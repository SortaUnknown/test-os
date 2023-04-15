#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};

entry_point!(kernel_start);

#[test_case]
fn test_breakpoint_exception()
{
    //invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

fn kernel_start(_boot_info: &'static BootInfo) -> !
{
    test_os::init();
    test_main();
    
    test_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    test_os::test_panic_handler(info);
}