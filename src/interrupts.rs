use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
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

#[cfg(test)]
mod test {
    #[test_case]
    fn test_breakpoint_exception() {
        x86_64::instructions::interrupts::int3();
    }
}
