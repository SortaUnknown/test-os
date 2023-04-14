*3.2 Paging Implementation*
enforced demarcation of unsafetyness in unsafe functions via `#[deny(unsafe_op_in_unsafe_fn)]`

*3.3 Heap Allocation*
`#[alloc_error_handler]` section appears to have been entirely superseded by Rust updates, so it was not implemented

*3.4 Allocator Designs*
minimal deviation in Fixed-Size Block Allocator implementation, due to changes in Rust syntax
bump allocator implementation is interesting, but useless. prune in future