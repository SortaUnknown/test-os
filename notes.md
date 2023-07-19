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
investigate replacing conquer_once oncecell with core oncecell (currently non-thread safe)
the ability to do QEMU tests appears to have been broken with the most recent bootloader, with no proper replacement currently (see https://github.com/rust-osdev/bootloader/issues/366)

**Rust lang dev Points of Interest**
std-Aware Cargo (https://github.com/rust-lang/wg-cargo-std-aware): Working Group behind built-std, currently very minimal implementation, ideally will allow to build custom implementations of the std library

**Unstable features Reference**
x86-interrupt: Lang 40180 (https://github.com/rust-lang/rust/issues/40180)
const mut refs: Lang 57349 (https://github.com/rust-lang/rust/issues/57349)
custom test frameworks: Lang 50297 (https://github.com/rust-lang/rust/issues/50297) (closed and with no proper replacement)