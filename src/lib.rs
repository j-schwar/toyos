#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

pub mod allocator;
pub mod gdt;
pub mod interrupts;
pub mod mem;
pub mod serial;
pub mod task;
pub mod vga;

/// Initializes the kernel.
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    interrupts::init_hw_interrupts();
}

/// Halts the CPU causing it to enter a sleep state until the next interrupt
/// arrives.
pub fn hlt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Exit codes that can be passed to [exit_qemu].
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QemuExitCode {
    Success = 0x10,
    Error = 0x11,
}

/// Signals QEMU to exit from the guest operating system.
///
/// In order for this function to work, the `isa-debug-exit` devices must be
/// enabled. This can be done by passing
/// `-device isa-debug-exit,iobase=0xf4,iosize=0x4` to QEMU. Note that the port
/// number should be `0xf4` with a size of 4 bytes.
///
/// If setup correctly, QEMU will exit with a code of `exit_code << 1 | 1`.
/// [QemuExitCode] defines exit codes which integrate with the test framework
/// where an exit code of `0x10 << 1 | 1 = 33` indicates success. See the
/// additional configuration in Cargo.toml for how cargo test integrates with
/// QEMU and the test framework.
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;

    const ISA_DEBUG_EXIT_PORT: u16 = 0xf4;
    unsafe {
        let mut port = Port::new(ISA_DEBUG_EXIT_PORT);
        port.write(exit_code as u32);
    }

    hlt();
}

/// Trait for test cases.
pub trait Testable {
    /// Invokes this test case.
    fn run(&self) -> ();
}

impl<F> Testable for F
where
    F: Fn(),
{
    /// Invokes the test case printing test information to the serial port.
    fn run(&self) -> () {
        serial_print!("{}... ", core::any::type_name::<F>());
        self();
        serial_println!("[ok]");
    }
}

/// Invokes a given set of test cases.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

/// Panic handler implementation suitable for tests.
///
/// Integration tests may delegate panic handling to this function by calling
/// it in their panic handler:
///
/// ```no_run
/// #[panic_handler]
/// fn panic(info: &PanicInfo) -> ! {
///     toyos::test_panic_handler(info)
/// }
/// ```
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Error);
}

#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

/// Entry point for `cargo test` when testing this crate.
#[cfg(test)]
fn test_kernel_main(_boot_info: &'static bootloader::BootInfo) -> ! {
    init();
    test_main();
    hlt();
}

/// Panic handler for `cargo test`.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
