#![no_std]
#![no_main]

//
// Following the blob by Philipp Oppermann: https://os.phil-opp.com
//

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
