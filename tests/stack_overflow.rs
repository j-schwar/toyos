#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use lazy_static::lazy_static;
use toyos::{serial_print, serial_println, QemuExitCode};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("stack_overflow::stack_overflow... ");

    // Initialize global descriptor table.
    toyos::gdt::init();

    // Initialize test interrupt descriptor table.
    init_test_idt();

    // Trigger a stack overflow.
    stack_overflow();

    // This line should never be reached as the double fault exception handler
    // should catch the exception completing the test and exiting QEMU.
    panic!("execution continued after stack overflow");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    toyos::test_panic_handler(info)
}

// Trigger a stack overflow with infinite recursion.
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read(); // Prevent tail-recursion optimization.
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        use toyos::gdt::DOUBLE_FAULT_IST_INDEX;

        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

fn init_test_idt() {
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    toyos::exit_qemu(QemuExitCode::Success);
}
