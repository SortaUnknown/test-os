#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use test_os::serial_println;
use test_os::serial_print;
use test_os::exit_qemu;
use test_os::QemuExitCode;
use bootloader::BootInfo;
use bootloader::entry_point;

entry_point!(kernel_start);

fn should_fail()
{
    serial_print!("should_panic::should_fail... ");
    assert_eq!(0, 1);
}

fn kernel_start(_boot_info: &'static BootInfo) -> !
{
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    
    test_os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    test_os::test_panic_handler(info);
}