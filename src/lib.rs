// SPDX-License-Identifier: Apache-2.0 OR MIT

/*!
A type that asserts that the underlying type is not moved after being
[pinned][pin] and mutably accessed.

This is a rewrite of [futures-test]'s `AssertUnmoved` to allow use in more
use cases. This also supports traits other than [futures][futures03].

Many of the changes made in this project are also reflected upstream:
[rust-lang/futures-rs#2148], [rust-lang/futures-rs#2208]

## Examples

An example of detecting incorrect [`Pin::new_unchecked`] use (**should panic**):

```rust,should_panic
use std::pin::Pin;

use assert_unmoved::AssertUnmoved;
use futures_util::{
    future::{self, Future},
    task::{noop_waker, Context},
};

let waker = noop_waker();
let mut cx = Context::from_waker(&waker);

// First we allocate the future on the stack and poll it.
let mut future = AssertUnmoved::new(future::pending::<()>());
let pinned_future = unsafe { Pin::new_unchecked(&mut future) };
assert!(pinned_future.poll(&mut cx).is_pending());

// Next we move it back to the heap and poll it again. This second call
// should panic (as the future is moved).
let mut boxed_future = Box::new(future);
let pinned_boxed_future = unsafe { Pin::new_unchecked(&mut *boxed_future) };
let _ = pinned_boxed_future.poll(&mut cx).is_pending();
```

An example of detecting incorrect [`StreamExt::next`] implementation (**should panic**):

```rust,should_panic
# #[cfg(not(feature = "futures03"))]
# fn main() { unimplemented!() }
# #[cfg(feature = "futures03")]
# fn main() {
use std::pin::Pin;

use assert_unmoved::AssertUnmoved;
use futures_util::{
    future::Future,
    stream::{self, Stream},
    task::{noop_waker, Context, Poll},
};

struct Next<'a, S: Stream>(&'a mut S);

impl<S: Stream> Future for Next<'_, S> {
    type Output = Option<S::Item>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // This is `Pin<&mut Type>` to `Pin<Field>` projection and is unsound
        // if `S` is not `Unpin` (you can move `S` after `Next` dropped).
        //
        // The correct projection is `Pin<&mut Type>` to `Pin<&mut Field>`.
        // In `Next`, it is `Pin<&mut Next<'_, S>>` to `Pin<&mut &mut S>`,
        // and it needs to add `S: Unpin` bounds to convert `Pin<&mut &mut S>`
        // to `Pin<&mut S>`.
        let stream: Pin<&mut S> = unsafe { self.map_unchecked_mut(|f| f.0) };
        stream.poll_next(cx)
    }
}

let waker = noop_waker();
let mut cx = Context::from_waker(&waker);

let mut stream = AssertUnmoved::new(stream::pending::<()>());

{
    let next = Next(&mut stream);
    let mut pinned_next = Box::pin(next);
    assert!(pinned_next.as_mut().poll(&mut cx).is_pending());
}

// Move stream to the heap.
let mut boxed_stream = Box::pin(stream);

let next = Next(&mut boxed_stream);
let mut pinned_next = Box::pin(next);
// This should panic (as the future is moved).
let _ = pinned_next.as_mut().poll(&mut cx).is_pending();
# }
```

## Optional features

- **`futures03`** — Implements [futures v0.3][futures03] traits for assert-unmoved types.
- **`tokio1`** — Implements [tokio v1][tokio1] traits for assert-unmoved types.

Note: The MSRV when these features are enabled depends on the MSRV of these crates.

[`Pin::new_unchecked`]: https://doc.rust-lang.org/std/pin/struct.Pin.html#method.new_unchecked
[`StreamExt::next`]: https://docs.rs/futures/0.3/futures/stream/trait.StreamExt.html#method.next
[futures-test]: https://docs.rs/futures-test
[futures03]: https://docs.rs/futures/0.3
[pin]: https://doc.rust-lang.org/std/pin/index.html
[rust-lang/futures-rs#2148]: https://github.com/rust-lang/futures-rs/pull/2148
[rust-lang/futures-rs#2208]: https://github.com/rust-lang/futures-rs/pull/2208
[tokio1]: https://docs.rs/tokio/1
*/

#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_variables)
    )
))]
#![warn(
    rust_2018_idioms,
    single_use_lifetimes,
    unreachable_pub,
    clippy::pedantic,
    // Lints that may help when writing public library.
    missing_debug_implementations,
    missing_docs,
    clippy::alloc_instead_of_core,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::impl_trait_in_params,
    // clippy::missing_inline_in_public_items,
    // clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    // Lints that may help when writing unsafe code.
    improper_ctypes,
    improper_ctypes_definitions,
    // unsafe_op_in_unsafe_fn, // set conditionally since it requires Rust 1.52
    clippy::as_ptr_cast_mut,
    clippy::default_union_representation,
    clippy::inline_asm_x86_att_syntax,
    clippy::trailing_empty_array,
    clippy::transmute_undefined_repr,
    clippy::undocumented_unsafe_blocks,
)]
#![cfg_attr(test, warn(unsafe_op_in_unsafe_fn))] // unsafe_op_in_unsafe_fn requires Rust 1.52
#![cfg_attr(not(test), allow(unused_unsafe))]
#![allow(
    clippy::must_use_candidate,
    clippy::borrow_as_ptr, // https://github.com/rust-lang/rust-clippy/issues/8286
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(test)]
#[path = "gen/assert_impl.rs"]
mod assert_impl;

mod assert_unmoved;
pub use crate::assert_unmoved::AssertUnmoved;
