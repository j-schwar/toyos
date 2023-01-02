use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// Initializes a new offset page table.
///
/// # Safety
///
/// This function is unsafe because the caller must guarantee that the complete
/// physical memory is mapped at the given `physical_memory_offset`. Also, this
/// function must only be called once to avoid aliasing `&mut` references which
/// is undefined behavior.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let l4_table = active_level_4_page_table(physical_memory_offset);
    OffsetPageTable::new(l4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 page table.
///
/// # Safety
///
/// This function is unsafe because the caller must guarantee that the complete
/// physical memory is mapped at the given `physical_memory_offset`. Also, this
/// function must only be called once to avoid aliasing `&mut` references which
/// is undefined behavior.
unsafe fn active_level_4_page_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub fn map_to_example(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    unsafe { mapper.map_to(page, frame, flags, frame_allocator) }
        .expect("map_to failed")
        .flush();
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

/// A [FrameAllocator] that returns usable frames from the boot leader's
/// memory map.
pub struct BootInfoFrameAllocator<'a> {
    memory_map: &'a MemoryMap,
    next: usize,
}

impl<'a> BootInfoFrameAllocator<'a> {
    /// Constructs a frame allocator from a memory map.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must ensure that the given
    /// memory map is valid. The main requirement is that all frames marked as
    /// `USEABLE` are really unused.
    pub unsafe fn new(memory_map: &'a MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over all useable frames specified in the memory map.
    fn useable_frames(&self) -> impl Iterator<Item = PhysFrame> + 'a {
        let regions = self.memory_map.iter();
        regions
            .filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| r.range.start_addr()..r.range.end_addr())
            .flat_map(|r| r.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl<'a> FrameAllocator<Size4KiB> for BootInfoFrameAllocator<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.useable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
