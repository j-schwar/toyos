#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(toyos::test_runner)]
#![reexport_test_harness_main = "test_main"]

//
// Following the blob by Philipp Oppermann: https://os.phil-opp.com
//

use core::panic::PanicInfo;

use toyos::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    #[cfg(test)]
    test_main();

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
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
