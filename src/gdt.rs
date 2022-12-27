//! The `gdt` module defines the Global Descriptor Table (GDT) which, on older
//! systems was used for segmentation. More importantly, it's used to setup
//! the Task State Segment (TSS) which contains the Interrupt Stack Table (IST)
//! which allows stack swapping when calling interrupt handlers.
//! 
//! See: https://os.phil-opp.com/double-fault-exceptions/

use lazy_static::lazy_static;
use x86_64::registers::segmentation::{Segment, CS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// Interrupt Stack Table (IST) index for the double fault handler stack.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct Segments {
    code_segment: SegmentSelector,
    tss_segment: SegmentSelector,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Setup a dedicated stack for the `double fault` exception handler.
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            // TODO: replace with proper stack allocation
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        tss
    };

    static ref GDT: (GlobalDescriptorTable, Segments) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_segment = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_segment = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Segments { code_segment, tss_segment })
    };
}

/// Initializes this module by creating and loading the global descriptor
/// table and task state segment.
pub fn init() {
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        // Set the code segment register.
        CS::set_reg(GDT.1.code_segment);

        // Load the task state segment.
        load_tss(GDT.1.tss_segment);
    }
}
