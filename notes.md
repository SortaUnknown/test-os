**3.2 Paging Implementation**
enforced demarcation of unsafetyness in unsafe functions via `#[deny(unsafe_op_in_unsafe_fn)]`

**3.3 Heap Allocation**
`#[alloc_error_handler]` section appears to have been entirely superseded by Rust updates, so it was not implemented

**3.4 Allocator Designs**
minimal deviation in Fixed-Size Block Allocator implementation, due to changes in Rust syntax
bump allocator implementation is interesting, but useless. prune in future

**4.1 Async/Await**
destructuring no longer required
simple executor is also interesting, but useless. prune in future

**Updating deps**
for fixed size allocator, usize can be cast to ptr safely
volatile crate replaced with core::ptr::read_volatile() and write_volatile()
bootloader full update requires major crate system rewrite, including dropping bootimage dep and changing the tests system, pushing back for now

**Compiling for x86_64-unknown-none**
compile results in panic "Mapping(PageAlreadyMapped(PhysFrame[4KiB]))"