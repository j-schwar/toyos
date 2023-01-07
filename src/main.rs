#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(toyos::test_runner)]
#![reexport_test_harness_main = "test_main"]

//
// Following the blob by Philipp Oppermann: https://os.phil-opp.com
//

extern crate alloc;

use core::panic::PanicInfo;

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use bootloader::BootInfo;
use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
use toyos::{
    mem::BootInfoFrameAllocator,
    println,
    task::{simple_executor::SimpleExecutor, Task},
};
use x86_64::VirtAddr;

bootloader::entry_point!(kernel_main);

const VERSION_MAJOR: u32 = pkg_version_major!();
const VERSION_MINOR: u32 = pkg_version_minor!();
const VERSION_PATCH: u32 = pkg_version_patch!();

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!(
        "Toy-OS version {}.{}.{}",
        VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH
    );

    toyos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { toyos::mem::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::new(&boot_info.memory_map) };

    toyos::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.run();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");

    toyos::hlt();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    toyos::hlt();
}

/// Panic handler for `cargo test`.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    toyos::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
