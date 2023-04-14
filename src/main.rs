#![no_std] //don't link Rust std
#![no_main] //disable all language-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(test_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::boxed::Box;
use bootloader::BootInfo;
use bootloader::entry_point;
use x86_64::VirtAddr;
use test_os::println;
use test_os::memory;
use test_os::memory::BootInfoFrameAllocator;
use test_os::allocator;

entry_point!(kernel_start);

//kernel entry point
fn kernel_start(boot_info: &'static BootInfo) -> !
{
    println!("Hello World{}", "!");

    test_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe{memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe{BootInfoFrameAllocator::init(&boot_info.memory_map)};

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let heap_value = Box::new(41);
    println!("heap value at {:p}", heap_value);

    #[cfg(test)]
    test_main();

    println!("End of Kernel");
    
    test_os::hlt_loop();
}

//default panic
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    println!("{}", info);

    test_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    test_os::test_panic_handler(info);
}