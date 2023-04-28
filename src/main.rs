#![no_std] //don't link Rust std
#![no_main] //disable all language-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(test_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader_api::info::MemoryRegions;
use bootloader_api::{BootInfo, entry_point};
use bootloader_api::config::{BootloaderConfig, Mapping};
use x86_64::VirtAddr;
use test_os::println;
use test_os::FRAME_BUFFER;
use test_os::memory;
use test_os::memory::BootInfoFrameAllocator;
use test_os::allocator;
use test_os::task::Task;
use test_os::task::executor::Executor;
use test_os::task::keyboard::print_keypresses;
use conquer_once::spin::OnceCell;
use spin::Mutex;

pub static BOOTLOADER_CONFIG: BootloaderConfig =
{
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_start, config = &BOOTLOADER_CONFIG);

//kernel entry point
fn kernel_start(boot_info: &'static mut BootInfo) -> !
{
    FRAME_BUFFER.init_once(|| boot_info.framebuffer.into_option().expect("framebuffer err"));
    println!("Hello World{}", "!");

    test_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().expect("physmem err"));
    let mut mapper = unsafe{memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe{BootInfoFrameAllocator::init(&boot_info.memory_regions)};

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    println!("End of Kernel");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(print_keypresses()));
    executor.run();
}

async fn async_number() -> u8
{
    42
}

async fn example_task()
{
    let number = async_number().await;
    println!("async number: {}", number);
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