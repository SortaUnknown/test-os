#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader_api::{BootInfo, entry_point};

entry_point!(kernel_start);

#[test_case]
fn test_breakpoint_exception()
{
    //invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}

fn kernel_start(_boot_info: &'static mut BootInfo) -> !
{
    test_kernel::init();
    test_main();
    
    test_kernel::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    test_kernel::test_panic_handler(info);
}