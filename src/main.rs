#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(toyos::test_runner)]
#![reexport_test_harness_main = "test_main"]

//
// Following the blob by Philipp Oppermann: https://os.phil-opp.com
//

use core::panic::PanicInfo;

use bootloader::BootInfo;
use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
use toyos::println;
use x86_64::{
    structures::paging::{Page, Translate},
    VirtAddr,
};

bootloader::entry_point!(kernel_main);

const VERSION_MAJOR: u32 = pkg_version_major!();
const VERSION_MINOR: u32 = pkg_version_minor!();
const VERSION_PATCH: u32 = pkg_version_patch!();

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!(
        "Toy-OS version {}.{}.{}",
        VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH
    );

    toyos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut page_mapper = unsafe { toyos::mem::init(phys_mem_offset) };

    let mut frame_allocator = unsafe {
        toyos::mem::BootInfoFrameAllocator::new(&boot_info.memory_map)
    };
    let addr = 0xdeadbeef000;
    let page = Page::containing_address(VirtAddr::new(addr));
    toyos::mem::map_to_example(page, &mut page_mapper, &mut frame_allocator);

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // Re-mapped to the vga buffer page
        addr,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &addr in &addresses {
        let virt = VirtAddr::new(addr);
        let phys = page_mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

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
