use std::{future::Future, pin::Pin, task::Context};

use assert_unmoved::AssertUnmoved;
use futures_util::{future::pending, task::noop_waker};

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
