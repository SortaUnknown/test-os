#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use test_os::serial_print;
use test_os::serial_println;
use test_os::QemuExitCode;
use test_os::exit_qemu;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use lazy_static::lazy_static;
use bootloader::BootInfo;
use bootloader::entry_point;

entry_point!(kernel_start);

lazy_static!
{
    static ref TEST_IDT: InterruptDescriptorTable =
    {
        let mut idt = InterruptDescriptorTable::new();

        unsafe{idt.double_fault.set_handler_fn(test_double_fault_handler).set_stack_index(test_os::gdt::DOUBLE_FAULT_IST_INDEX);}

        idt
    };
}

fn kernel_start(_boot_info: &'static BootInfo) -> !
{
    serial_print!("stack_overflow::stack_overflow... ");

    test_os::gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    test_os::test_panic_handler(info);
}

pub fn init_test_idt()
{
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(_stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);

    test_os::hlt_loop();
}

#[allow(unconditional_recursion)]
fn stack_overflow()
{
    stack_overflow();
    volatile::Volatile::new(0).read();
}