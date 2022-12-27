//! The `interrupts` module defines interrupt handlers for CPU faults.
//!
//! The [init_idt] function can be used to initialize the interrupt descriptor
//! table during the kernel's initial boot sequence.
//!
//! See https://wiki.osdev.org/Exceptions for more info on CPU exceptions.

use crate::{gdt::DOUBLE_FAULT_IST_INDEX, println};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);

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

        idt
    };
}

/// Initializes the x86_64 interrupt descriptor table.
pub fn init_idt() {
    IDT.load();
}

/// Handler for the breakpoint CPU exception.
///
/// A breakpoint exception is triggered when the CPU executes a `int3`
/// instruction.
///
/// See: https://wiki.osdev.org/Exceptions#Breakpoint
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
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

#[cfg(test)]
mod test {
    #[test_case]
    fn test_breakpoint_exception() {
        x86_64::instructions::interrupts::int3();
    }
}
