use assert_unmoved::AssertUnmoved;
use futures_util::{future::pending, task::noop_waker};
use static_assertions::{assert_impl_all, assert_not_impl_all};
use std::{future::Future, pin::Pin, task::Context};

assert_impl_all!(*const (): Unpin);
assert_not_impl_all!(AssertUnmoved<()>: Unpin);
assert_impl_all!(AssertUnmoved<()>: Send, Sync);
assert_not_impl_all!(AssertUnmoved<*const ()>: Send, Sync);

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
