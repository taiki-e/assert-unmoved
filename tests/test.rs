use assert_unmoved::AssertUnmoved;
use futures_util::{future::pending, task::noop_waker};
use static_assertions::{assert_impl_all, assert_not_impl_all};
use std::{future::Future, pin::Pin, task::Context};

assert_impl_all!(*const (): Unpin);
assert_not_impl_all!(AssertUnmoved<()>: Unpin);
assert_impl_all!(AssertUnmoved<()>: Send, Sync);
assert_not_impl_all!(AssertUnmoved<*const ()>: Send, Sync);

assert_impl_all!(AssertUnmoved<Pin<Box<dyn Future<Output = ()>>>>: Future<Output = ()>);

#[cfg(feature = "futures03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn futures_core::FusedFuture<Output = ()>>>>: futures_core::FusedFuture<Output = ()>);
#[cfg(feature = "futures03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn futures_core::Stream<Item = ()>>>>: futures_core::Stream<Item = ()>);
#[cfg(feature = "futures03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn futures_core::FusedStream<Item = ()>>>>: futures_core::FusedStream<Item = ()>);
#[cfg(feature = "futures03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn futures_sink::Sink<(), Error = ()>>>>: futures_sink::Sink<(), Error = ()>);
#[cfg(feature = "futures03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn futures_io::AsyncRead>>>: futures_io::AsyncRead);
#[cfg(feature = "futures03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn futures_io::AsyncWrite>>>: futures_io::AsyncWrite);
#[cfg(feature = "futures03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn futures_io::AsyncSeek>>>: futures_io::AsyncSeek);
#[cfg(feature = "futures03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn futures_io::AsyncBufRead>>>: futures_io::AsyncBufRead);

#[cfg(feature = "tokio02")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio02_crate::io::AsyncRead>>>: tokio02_crate::io::AsyncRead);
#[cfg(feature = "tokio02")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio02_crate::io::AsyncWrite>>>: tokio02_crate::io::AsyncWrite);
#[cfg(feature = "tokio02")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio02_crate::io::AsyncSeek>>>: tokio02_crate::io::AsyncSeek);
#[cfg(feature = "tokio02")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio02_crate::io::AsyncBufRead>>>: tokio02_crate::io::AsyncBufRead);

#[cfg(feature = "tokio03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio03_crate::io::AsyncRead>>>: tokio03_crate::io::AsyncRead);
#[cfg(feature = "tokio03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio03_crate::io::AsyncWrite>>>: tokio03_crate::io::AsyncWrite);
#[cfg(feature = "tokio03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio03_crate::io::AsyncSeek>>>: tokio03_crate::io::AsyncSeek);
#[cfg(feature = "tokio03")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio03_crate::io::AsyncBufRead>>>: tokio03_crate::io::AsyncBufRead);

#[cfg(feature = "tokio1")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio1_crate::io::AsyncRead>>>: tokio1_crate::io::AsyncRead);
#[cfg(feature = "tokio1")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio1_crate::io::AsyncWrite>>>: tokio1_crate::io::AsyncWrite);
#[cfg(feature = "tokio1")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio1_crate::io::AsyncSeek>>>: tokio1_crate::io::AsyncSeek);
#[cfg(feature = "tokio1")]
assert_impl_all!(AssertUnmoved<Pin<Box<dyn tokio1_crate::io::AsyncBufRead>>>: tokio1_crate::io::AsyncBufRead);

#[test]
fn dont_panic_when_not_polled() {
    // This shouldn't panic.
    let future = AssertUnmoved::new(pending::<()>());
    drop(future);
}

#[test]
#[should_panic(expected = "AssertUnmoved moved between get_pin_mut calls")]
fn dont_double_panic() {
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
    unsafe { x.map_unchecked_mut(|x| &mut x.0) }.as_pin_mut().unwrap().get_pin_mut();
}
