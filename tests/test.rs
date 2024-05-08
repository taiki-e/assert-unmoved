// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(clippy::let_underscore_future, clippy::undocumented_unsafe_blocks)]

use std::{
    future::{pending, Future},
    pin::Pin,
    task::Context,
};

use assert_unmoved::AssertUnmoved;
use futures::task::noop_waker;

#[test]
fn do_not_panic_when_not_polled() {
    // This shouldn't panic.
    let future = AssertUnmoved::new(pending::<()>());
    drop(future);
}

#[test]
#[should_panic(expected = "AssertUnmoved moved between get_pin_mut calls")]
fn do_not_double_panic() {
    // This test should only panic, not abort the process.
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    // First we allocate the future on the stack and poll it.
    let mut future = AssertUnmoved::new(pending::<()>());
    let pinned_future = unsafe { Pin::new_unchecked(&mut future) };
    assert!(pinned_future.poll(&mut cx).is_pending());

    // Next we move it back to the heap and poll it again. This second call
    // should panic (as the future is moved), but we shouldn't panic again
    // whilst dropping `AssertUnmoved`.
    let mut future = Box::new(future);
    let pinned_boxed_future = unsafe { Pin::new_unchecked(&mut *future) };
    assert!(pinned_boxed_future.poll(&mut cx).is_pending());
}

#[test]
#[should_panic(expected = "AssertUnmoved moved before drop")]
fn moved_before_drop() {
    struct Test<T>(Option<T>);

    impl<T> Drop for Test<T> {
        fn drop(&mut self) {
            // This moves `T`.
            self.0.take();
        }
    }

    let mut x = Test(Some(AssertUnmoved::new(pending::<()>())));
    let x = unsafe { Pin::new_unchecked(&mut x) };
    // This `map_unchecked_mut` is unsound because `Test`'s destructor moves `T`.
    let _ = unsafe { x.map_unchecked_mut(|x| &mut x.0) }.as_pin_mut().unwrap().get_pin_mut();
}

#[test]
#[should_panic(expected = "AssertUnmoved moved after get_pin_mut call")]
fn misuse_get_mut() {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    let mut future = AssertUnmoved::new(pending::<()>());
    let pinned_future = unsafe { Pin::new_unchecked(&mut future) };
    assert!(pinned_future.poll(&mut cx).is_pending());

    let mut future = Box::new(future);
    let _ = future.get_mut();
}

pub mod assert_impl {
    use static_assertions::assert_impl_all as assert_impl;
    #[cfg(feature = "tokio02")]
    use tokio02_crate as tokio02;
    #[cfg(feature = "tokio03")]
    use tokio03_crate as tokio03;
    #[cfg(feature = "tokio1")]
    use tokio1_crate as tokio1;

    use crate::*;

    assert_impl!(AssertUnmoved<Pin<Box<dyn Future<Output = ()>>>>: Future<Output = ()>);

    #[cfg(feature = "futures03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn futures_core::FusedFuture<Output = ()>>>>: futures_core::FusedFuture<Output = ()>);
    #[cfg(feature = "futures03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn futures_core::Stream<Item = ()>>>>: futures_core::Stream<Item = ()>);
    #[cfg(feature = "futures03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn futures_core::FusedStream<Item = ()>>>>: futures_core::FusedStream<Item = ()>);
    #[cfg(feature = "futures03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn futures_sink::Sink<(), Error = ()>>>>: futures_sink::Sink<(), Error = ()>);
    #[cfg(feature = "futures03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn futures_io::AsyncRead>>>: futures_io::AsyncRead);
    #[cfg(feature = "futures03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn futures_io::AsyncWrite>>>: futures_io::AsyncWrite);
    #[cfg(feature = "futures03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn futures_io::AsyncSeek>>>: futures_io::AsyncSeek);
    #[cfg(feature = "futures03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn futures_io::AsyncBufRead>>>: futures_io::AsyncBufRead);

    #[cfg(feature = "tokio02")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio02::io::AsyncRead>>>: tokio02::io::AsyncRead);
    #[cfg(feature = "tokio02")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio02::io::AsyncWrite>>>: tokio02::io::AsyncWrite);
    #[cfg(feature = "tokio02")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio02::io::AsyncSeek>>>: tokio02::io::AsyncSeek);
    #[cfg(feature = "tokio02")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio02::io::AsyncBufRead>>>: tokio02::io::AsyncBufRead);

    #[cfg(feature = "tokio03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio03::io::AsyncRead>>>: tokio03::io::AsyncRead);
    #[cfg(feature = "tokio03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio03::io::AsyncWrite>>>: tokio03::io::AsyncWrite);
    #[cfg(feature = "tokio03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio03::io::AsyncSeek>>>: tokio03::io::AsyncSeek);
    #[cfg(feature = "tokio03")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio03::io::AsyncBufRead>>>: tokio03::io::AsyncBufRead);

    #[cfg(feature = "tokio1")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio1::io::AsyncRead>>>: tokio1::io::AsyncRead);
    #[cfg(feature = "tokio1")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio1::io::AsyncWrite>>>: tokio1::io::AsyncWrite);
    #[cfg(feature = "tokio1")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio1::io::AsyncSeek>>>: tokio1::io::AsyncSeek);
    #[cfg(feature = "tokio1")]
    assert_impl!(AssertUnmoved<Pin<Box<dyn tokio1::io::AsyncBufRead>>>: tokio1::io::AsyncBufRead);
}
