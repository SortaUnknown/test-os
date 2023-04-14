use linked_list_allocator::Heap;
use alloc::alloc::Layout;
use alloc::alloc::GlobalAlloc;
use core::ptr::null_mut;
use core::ptr::NonNull;
use core::mem;
use super::Locked;

/// The block sizes to use.
///
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct ListNode
{
    next: Option<&'static mut ListNode>
}

pub struct FixedSizeBlockAllocator
{
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: Heap
}

impl FixedSizeBlockAllocator
{
    /// Creates an empty FixedSizeBlockAllocator.
    pub const fn new() -> Self
    {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator{list_heads: [EMPTY; BLOCK_SIZES.len()], fallback_allocator: Heap::empty()}
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.
    #[deny(unsafe_op_in_unsafe_fn)]
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize)
    {
        unsafe{self.fallback_allocator.init(heap_start, heap_size);}
    }

    /// Allocates using the fallback allocator.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8
    {
        match self.fallback_allocator.allocate_first_fit(layout)
        {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => null_mut()
        }
    }

    /// Choose an appropriate block size for the given layout.
    ///
    /// Returns an index into the `BLOCK_SIZES` array.
    fn list_index(&self, layout: &Layout) -> Option<usize> //TODO: why the fuck does rustc want a &self parameter when this doesn't need it, research
    {
        BLOCK_SIZES.iter().position(|&s| s >= layout.size().max(layout.align()))
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator>
{
    #[deny(unsafe_op_in_unsafe_fn)]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8
    {
        let mut allocator = self.lock();
        match allocator.list_index(&layout)
        {
            Some(index) =>
            {
                match allocator.list_heads[index].take()
                {
                    Some(node) =>
                    {
                        allocator.list_heads[index] = node.next.take();
                        node  as *mut ListNode as *mut u8
                    }
                    None =>
                    {
                        let block_size = BLOCK_SIZES[index];
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => allocator.fallback_alloc(layout)
        }
    }

    #[deny(unsafe_op_in_unsafe_fn)]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout)
    {
        let mut allocator = self.lock();
        match allocator.list_index(&layout)
        {
            Some(index) =>
            {
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                let new_node_ptr = ptr as *mut ListNode;
                unsafe{new_node_ptr.write(ListNode{next: allocator.list_heads[index].take()});}
                allocator.list_heads[index] = Some(unsafe{&mut *new_node_ptr});
            }
            None =>
            {
                unsafe{allocator.fallback_allocator.deallocate(NonNull::new(ptr).unwrap(), layout);}
            }
        }
    }
}