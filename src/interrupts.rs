//! The `interrupts` module defines interrupt handlers for CPU faults and
//! hardware interrupts.
//!
//! The [init_idt] function can be used to initialize the interrupt descriptor
//! table during the kernel's initial boot sequence.
//!
//! See https://wiki.osdev.org/Exceptions for more info on CPU exceptions.
//! See https://os.phil-opp.com/hardware-interrupts/ for hardware interrupts.

use crate::{gdt::DOUBLE_FAULT_IST_INDEX, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

// Offset into the interrupt table for hardware interrupt handlers for the two
// programmable interrupt controllers (PICs). Positions 0x0 through 0x1f are
// reserved for CPU fault handlers.
const PIC_1_OFFSET: u8 = 0x20;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // CPU faults

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        unsafe {
            // Register the double fault handler and configure it to use a
            // dedicated stack.
            //
            // We need a separate stack for this handler because we can't
            // ensure that the current stack is in a valid state. For example,
            // if a page fault exception occurs due to a stack overflow, an
            // attempt is made to push a new stack frame onto the stack in
            // order to call the page fault exception handler. Since the stack
            // point is already invalid due to the stack overflow, another page
            // fault exception is triggered which triggers a double fault
            // exception. However, like before, a stack frame cannot be pushed
            // to call the double fault exception handler which triggers a
            // triple fault causing a hard reset. Having a dedicated stack for
            // the double fault handler ensures that it can always be called,
            // even if the original stack is borked.
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        // Hardware interrupts

        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);

        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);

        idt
    };

    static ref PICS: Mutex<ChainedPics> = Mutex::new(
        unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }
    );
}

/// Interrupt indices for the Intel 8259 interrupt controller.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    #[inline]
    fn as_usize(self) -> usize {
        self as u8 as usize
    }
}

/// Initializes the x86_64 interrupt descriptor table.
pub fn init_idt() {
    IDT.load();
}

/// Initializes the two programmable interrupt controllers (PICs) and enable
/// interrupts.
pub fn init_hw_interrupts() {
    unsafe {
        PICS.lock().initialize();
    }

    x86_64::instructions::interrupts::enable();
}

//
// MARK: Interrupt Handlers
//

/// Handler for the breakpoint CPU exception.
///
/// A breakpoint exception is triggered when the CPU executes a `int3`
/// instruction.
///
/// See: https://wiki.osdev.org/Exceptions#Breakpoint
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Handler for page fault CPU exceptions.
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("Stack Frame: {:#?}", stack_frame);
    crate::hlt();
}

/// Handler for double fault CPU exceptions.
///
/// Recovery from this handler is not permitted. As such, this function does
/// not return.
///
/// See: https://wiki.osdev.org/Exception#Double_Fault
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n, {:#?}", stack_frame);
}

/// Handler for timer interrupts.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

/// Handler for keyboard interrupts.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

#[cfg(test)]
mod test {
    #[test_case]
    fn test_breakpoint_exception() {
        x86_64::instructions::interrupts::int3();
    }
}
