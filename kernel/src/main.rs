#![no_std] //don't link Rust std
#![no_main] //disable all language-level entry points

extern crate alloc;

use core::panic::PanicInfo;
use bootloader_api::{BootInfo, entry_point};
use bootloader_api::config::{BootloaderConfig, Mapping};
use x86_64::VirtAddr;
use kernel::memory;
use kernel::memory::BootInfoFrameAllocator;
use kernel::allocator;
use kernel::task::Task;
use kernel::task::executor::Executor;
use log::{info, error};

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
    let fb_struct = (&mut boot_info.framebuffer).as_mut().unwrap();
    let fb_info = fb_struct.info();
    let raw_fb = fb_struct.buffer_mut();

    kernel::init_logger(raw_fb, fb_info);

    kernel::init();

    info!("Hello World{}", "!");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().expect("physmem err"));
    let mut mapper = unsafe{memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe{BootInfoFrameAllocator::init(&boot_info.memory_regions)};

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    info!("End of Kernel");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(kernel::task::keyboard::print_keypresses()));
    executor.run();
}

async fn async_number() -> u8
{
    42
}

async fn example_task()
{
    let number = async_number().await;
    info!("async number: {}", number);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> !
{
    error!("{}", info);

    kernel::hlt_loop();
}