# assert-unmoved

[![crates.io](https://img.shields.io/crates/v/assert-unmoved.svg?style=flat-square&logo=rust)](https://crates.io/crates/assert-unmoved)
[![docs.rs](https://img.shields.io/badge/docs.rs-assert--unmoved-blue?style=flat-square)](https://docs.rs/assert-unmoved)
[![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue.svg?style=flat-square)](#license)
[![rustc](https://img.shields.io/badge/rustc-1.37+-blue.svg?style=flat-square)](https://www.rust-lang.org)
[![build status](https://img.shields.io/github/workflow/status/taiki-e/assert-unmoved/CI/master?style=flat-square)](https://github.com/taiki-e/assert-unmoved/actions?query=workflow%3ACI+branch%3Amaster)

A type that asserts that the underlying type is not moved after being [pinned][pin]
and mutably accessed.

This is a rewrite of [futures-test]'s `AssertUnmoved` to allow use in more
use cases. This also supports traits other than [futures][futures03].

Many of the changes made in this project are also reflected upstream: [rust-lang/futures-rs#2148], [rust-lang/futures-rs#2208]

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
assert-unmoved = "0.1"
```

*Compiler support: requires rustc 1.37+*

## Examples

An example of using [`Pin::new_unchecked`] incorrectly (**should panic**):

```rust
use futures_util::{
    future::{self, Future},
    task::{noop_waker, Context},
};
use assert_unmoved::AssertUnmoved;
use std::pin::Pin;

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

An example of incorrect [`StreamExt::next`] implementation (**should panic**):

```rust
use futures_util::{
    future::Future,
    stream::{self, Stream},
    task::{noop_waker, Context, Poll},
};
use assert_unmoved::AssertUnmoved;
use std::pin::Pin;

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

* **`futures03`** — Implements [futures 0.3][futures03] traits for assert-unmoved types.
* **`tokio02`** — Implements [tokio 0.2][tokio02] traits for assert-unmoved types.
* **`tokio03`** — Implements [tokio 0.3][tokio03] traits for assert-unmoved types.

[`Pin::new_unchecked`]: https://doc.rust-lang.org/std/pin/struct.Pin.html#method.new_unchecked
[`StreamExt::next`]: https://docs.rs/futures/0.3/futures/stream/trait.StreamExt.html#method.next
[futures-test]: https://docs.rs/futures-test
[futures03]: https://docs.rs/futures/0.3
[pin]: https://doc.rust-lang.org/std/pin/index.html
[rust-lang/futures-rs#2148]: https://github.com/rust-lang/futures-rs/pull/2148
[rust-lang/futures-rs#2208]: https://github.com/rust-lang/futures-rs/pull/2208
[tokio02]: https://docs.rs/tokio/0.2
[tokio03]: https://docs.rs/tokio/0.3

## Related Projects

* [pin-project]: A crate for safe and ergonomic pin-projection.
* [pin-project-lite]: A lightweight version of pin-project written with declarative macros.

[pin-project]: https://github.com/taiki-e/pin-project
[pin-project-lite]: https://github.com/taiki-e/pin-project-lite

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
