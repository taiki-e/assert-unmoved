# assert-unmoved

[![crates.io](https://img.shields.io/crates/v/assert-unmoved?style=flat-square&logo=rust)](https://crates.io/crates/assert-unmoved)
[![docs.rs](https://img.shields.io/badge/docs.rs-assert--unmoved-blue?style=flat-square&logo=docs.rs)](https://docs.rs/assert-unmoved)
[![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)](#license)
[![msrv](https://img.shields.io/badge/msrv-1.46-blue?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![github actions](https://img.shields.io/github/actions/workflow/status/taiki-e/assert-unmoved/ci.yml?branch=main&style=flat-square&logo=github)](https://github.com/taiki-e/assert-unmoved/actions)

A type that asserts that the underlying type is not moved after being
[pinned][pin] and mutably accessed.

This is a rewrite of [futures-test]'s `AssertUnmoved` to allow use in more
use cases. This also supports traits other than [futures][futures03].

Many of the changes made in this project are also reflected upstream:
[rust-lang/futures-rs#2148], [rust-lang/futures-rs#2208]

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
assert-unmoved = "0.1"
```

## Examples

An example of detecting incorrect [`Pin::new_unchecked`] use (**should panic**):

```rust
use std::pin::Pin;

use assert_unmoved::AssertUnmoved;
use futures::{
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

```rust
use std::pin::Pin;

use assert_unmoved::AssertUnmoved;
use futures::{
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
```

## Optional features

- **`futures03`** — Implements [futures v0.3][futures03] traits for assert-unmoved types.
- **`tokio1`** — Implements [tokio v1][tokio1] traits for assert-unmoved types.

Note: The MSRV when these features are enabled depends on the MSRV of these crates.

[`Pin::new_unchecked`]: https://doc.rust-lang.org/std/pin/struct.Pin.html#method.new_unchecked
[`StreamExt::next`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html#method.next
[futures-test]: https://docs.rs/futures-test
[futures03]: https://docs.rs/futures/0.3
[pin]: https://doc.rust-lang.org/std/pin/index.html
[rust-lang/futures-rs#2148]: https://github.com/rust-lang/futures-rs/pull/2148
[rust-lang/futures-rs#2208]: https://github.com/rust-lang/futures-rs/pull/2208
[tokio1]: https://docs.rs/tokio/1

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
