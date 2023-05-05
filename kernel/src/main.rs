#![no_std] //don't link Rust std
#![no_main] //disable all language-level entry points
/*#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]*/

extern crate alloc;

use core::panic::PanicInfo;
use bootloader_api::{BootInfo, entry_point};
use bootloader_api::config::{BootloaderConfig, Mapping};
use x86_64::VirtAddr;
use kernel::{FRAME_BUFFER, println};
use kernel::memory;
use kernel::memory::BootInfoFrameAllocator;
use kernel::allocator;
use kernel::task::Task;
use kernel::task::executor::Executor;

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
    FRAME_BUFFER.init_once(|| boot_info.framebuffer.as_ref().expect("framebuffer err"));
    println!("Hello World{}", "!");

    kernel::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().expect("physmem err"));
    let mut mapper = unsafe{memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe{BootInfoFrameAllocator::init(&boot_info.memory_regions)};

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    /*#[cfg(test)]
    test_main();*/

    println!("End of Kernel");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    //executor.spawn(Task::new(print_keypresses()));
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
//#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    println!("{}", info);

    kernel::hlt_loop();
}

/*#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    kernel::test_panic_handler(info);
}*/