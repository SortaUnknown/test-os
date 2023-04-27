use x86_64::structures::paging::{PageTable, OffsetPageTable, PhysFrame, Size4KiB, FrameAllocator};
use x86_64::registers::control::Cr3;
use x86_64::{VirtAddr, PhysAddr};
use bootloader_api::info::{MemoryRegions, MemoryRegionKind};

#[deny(unsafe_op_in_unsafe_fn)]
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static>
{
    let level_4_table = unsafe{active_level_4_table(physical_memory_offset)};
    unsafe{OffsetPageTable::new(level_4_table, physical_memory_offset)}
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior)
#[deny(unsafe_op_in_unsafe_fn)]
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable
{
    let virt = physical_memory_offset + Cr3::read().0.start_address().as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe{&mut *page_table_ptr}
}

pub struct BootInfoFrameAllocator
{
    memory_map: &'static MemoryRegions,
    next: usize
}

impl BootInfoFrameAllocator
{
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryRegions) -> Self
    {
        BootInfoFrameAllocator{memory_map, next: 0}
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame>
    {
        let usable_regions = self.memory_map.iter().filter(|r| r.kind == MemoryRegionKind::Usable);
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator
{
    fn allocate_frame(&mut self) -> Option<PhysFrame>
    {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}