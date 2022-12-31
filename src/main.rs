#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(toyos::test_runner)]
#![reexport_test_harness_main = "test_main"]

//
// Following the blob by Philipp Oppermann: https://os.phil-opp.com
//

use core::panic::PanicInfo;

use pkg_version::{pkg_version_major, pkg_version_minor, pkg_version_patch};
use toyos::println;

const VERSION_MAJOR: u32 = pkg_version_major!();
const VERSION_MINOR: u32 = pkg_version_minor!();
const VERSION_PATCH: u32 = pkg_version_patch!();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!(
        "Toy-OS version {}.{}.{}",
        VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH
    );

    toyos::init();

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
