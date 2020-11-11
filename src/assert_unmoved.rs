use pin_project::{pin_project, pinned_drop};
use std::{
    future::Future,
    ops::Deref,
    pin::Pin,
    ptr,
    task::{Context, Poll},
    thread,
};

/// A type that asserts that the underlying type is not moved after being pinned
/// and mutably accessed.
///
/// See crate level documentation for details.
#[pin_project(PinnedDrop, !Unpin)]
#[derive(Debug)]
pub struct AssertUnmoved<T> {
    #[pin]
    inner: T,
    // Invariant: The pointer is never dereferenced.
    this_ptr: *const Self,
}

// Safety: Safe due to `this_ptr`'s invariant.
unsafe impl<T: Send> Send for AssertUnmoved<T> {}
unsafe impl<T: Sync> Sync for AssertUnmoved<T> {}

impl<T> AssertUnmoved<T> {
    /// Creates a new `AssertUnmoved`.
    pub const fn new(inner: T) -> Self {
        Self { inner, this_ptr: ptr::null() }
    }

    /// Gets a reference to the underlying type.
    ///
    /// Unlike [`get_mut`](AssertUnmoved::get_mut) method, this method can always called.
    ///
    /// You can also access the underlying type via [`Deref`] impl.
    pub const fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the underlying type.
    ///
    /// Note that this method can only be called before pinned since
    /// `AssertUnmoved` is `!Unpin` (this is guaranteed by the type system!).
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Gets a pinned mutable reference to the underlying type.
    ///
    /// # Panics
    ///
    /// Panics if this `AssertUnmoved` moved after being pinned and mutably accessed.
    ///
    /// # Examples
    ///
    /// Implement own [`Stream`] trait for `AssertUnmoved`.
    ///
    /// ```rust
    /// use assert_unmoved::AssertUnmoved;
    /// use std::{
    ///     pin::Pin,
    ///     task::{Context, Poll},
    /// };
    ///
    /// pub trait MyStream {
    ///     type Item;
    ///
    ///     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;
    /// }
    ///
    /// impl<S: MyStream> MyStream for AssertUnmoved<S> {
    ///     type Item = S::Item;
    ///
    ///     fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    ///         self.get_pin_mut().poll_next(cx)
    ///     }
    /// }
    /// ```
    ///
    /// [`Stream`]: https://docs.rs/futures/0.3/futures/stream/trait.Stream.html
    pub fn get_pin_mut(mut self: Pin<&mut Self>) -> Pin<&mut T> {
        let cur_this = &*self as *const Self;
        if self.this_ptr.is_null() {
            // First time being pinned and mutably accessed.
            *self.as_mut().project().this_ptr = cur_this;
        } else {
            assert_eq!(self.this_ptr, cur_this, "AssertUnmoved moved between get_pin_mut calls");
        }
        self.project().inner
    }
}

#[pinned_drop]
impl<T> PinnedDrop for AssertUnmoved<T> {
    /// # Panics
    ///
    /// Panics if this `AssertUnmoved` moved after being pinned and mutably accessed.
    fn drop(self: Pin<&mut Self>) {
        // If the thread is panicking then we can't panic again as that will
        // cause the process to be aborted.
        if !thread::panicking() && !self.this_ptr.is_null() {
            let cur_this = &*self as *const Self;
            assert_eq!(self.this_ptr, cur_this, "AssertUnmoved moved before drop");
        }
    }
}

impl<T> Deref for AssertUnmoved<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_ref()
    }
}

impl<T> From<T> for AssertUnmoved<T> {
    /// Converts a `T` into a `AssertUnmoved<T>`.
    ///
    /// This is equivalent to [`AssertUnmoved::new`].
    fn from(inner: T) -> Self {
        Self::new(inner)
    }
}

impl<T: Default> Default for AssertUnmoved<T> {
    /// Creates a new `AssertUnmoved`, with the default value for `T`.
    ///
    /// This is equivalent to [`AssertUnmoved::new(T::default())`](AssertUnmoved::new).
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<F: Future> Future for AssertUnmoved<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_pin_mut().poll(cx)
    }
}

#[cfg(feature = "futures03")]
mod futures03 {
    use super::AssertUnmoved;
    use futures_core::{
        future::FusedFuture,
        stream::{FusedStream, Stream},
    };
    use futures_io as io;
    use futures_sink::Sink;
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    impl<F: FusedFuture> FusedFuture for AssertUnmoved<F> {
        fn is_terminated(&self) -> bool {
            self.get_ref().is_terminated()
        }
    }

    impl<S: Stream> Stream for AssertUnmoved<S> {
        type Item = S::Item;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.get_pin_mut().poll_next(cx)
        }
    }

    impl<S: FusedStream> FusedStream for AssertUnmoved<S> {
        fn is_terminated(&self) -> bool {
            self.get_ref().is_terminated()
        }
    }

    impl<S: Sink<Item>, Item> Sink<Item> for AssertUnmoved<S> {
        type Error = S::Error;

        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.get_pin_mut().poll_ready(cx)
        }

        fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
            self.get_pin_mut().start_send(item)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.get_pin_mut().poll_flush(cx)
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.get_pin_mut().poll_close(cx)
        }
    }

    impl<R: io::AsyncRead> io::AsyncRead for AssertUnmoved<R> {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_read(cx, buf)
        }

        fn poll_read_vectored(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &mut [io::IoSliceMut<'_>],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_read_vectored(cx, bufs)
        }
    }

    impl<W: io::AsyncWrite> io::AsyncWrite for AssertUnmoved<W> {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_write(cx, buf)
        }

        fn poll_write_vectored(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[io::IoSlice<'_>],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_write_vectored(cx, bufs)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_flush(cx)
        }

        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_close(cx)
        }
    }

    impl<S: io::AsyncSeek> io::AsyncSeek for AssertUnmoved<S> {
        fn poll_seek(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            pos: io::SeekFrom,
        ) -> Poll<io::Result<u64>> {
            self.get_pin_mut().poll_seek(cx, pos)
        }
    }

    impl<R: io::AsyncBufRead> io::AsyncBufRead for AssertUnmoved<R> {
        fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
            self.get_pin_mut().poll_fill_buf(cx)
        }

        fn consume(self: Pin<&mut Self>, amt: usize) {
            self.get_pin_mut().consume(amt)
        }
    }
}

#[cfg(feature = "tokio02")]
mod tokio02 {
    use super::AssertUnmoved;
    use bytes05::{Buf, BufMut};
    use std::{
        io,
        mem::MaybeUninit,
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio02_crate::io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite};

    impl<R: AsyncRead> AsyncRead for AssertUnmoved<R> {
        unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [MaybeUninit<u8>]) -> bool {
            self.get_ref().prepare_uninitialized_buffer(buf)
        }

        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_read(cx, buf)
        }

        fn poll_read_buf<B: BufMut>(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut B,
        ) -> Poll<io::Result<usize>>
        where
            Self: Sized,
        {
            self.get_pin_mut().poll_read_buf(cx, buf)
        }
    }

    impl<W: AsyncWrite> AsyncWrite for AssertUnmoved<W> {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_write(cx, buf)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_flush(cx)
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_shutdown(cx)
        }

        fn poll_write_buf<B: Buf>(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut B,
        ) -> Poll<Result<usize, io::Error>>
        where
            Self: Sized,
        {
            self.get_pin_mut().poll_write_buf(cx, buf)
        }
    }

    impl<S: AsyncSeek> AsyncSeek for AssertUnmoved<S> {
        fn start_seek(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            pos: io::SeekFrom,
        ) -> Poll<io::Result<()>> {
            self.get_pin_mut().start_seek(cx, pos)
        }

        fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
            self.get_pin_mut().poll_complete(cx)
        }
    }

    impl<R: AsyncBufRead> AsyncBufRead for AssertUnmoved<R> {
        fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
            self.get_pin_mut().poll_fill_buf(cx)
        }

        fn consume(self: Pin<&mut Self>, amt: usize) {
            self.get_pin_mut().consume(amt)
        }
    }
}

#[cfg(feature = "tokio03")]
mod tokio03 {
    use super::AssertUnmoved;
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio03::io;

    impl<R: io::AsyncRead> io::AsyncRead for AssertUnmoved<R> {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut io::ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_read(cx, buf)
        }
    }

    impl<W: io::AsyncWrite> io::AsyncWrite for AssertUnmoved<W> {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_write(cx, buf)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_flush(cx)
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_shutdown(cx)
        }
    }

    impl<S: io::AsyncSeek> io::AsyncSeek for AssertUnmoved<S> {
        fn start_seek(self: Pin<&mut Self>, pos: io::SeekFrom) -> io::Result<()> {
            self.get_pin_mut().start_seek(pos)
        }

        fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
            self.get_pin_mut().poll_complete(cx)
        }
    }

    impl<R: io::AsyncBufRead> io::AsyncBufRead for AssertUnmoved<R> {
        fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
            self.get_pin_mut().poll_fill_buf(cx)
        }

        fn consume(self: Pin<&mut Self>, amt: usize) {
            self.get_pin_mut().consume(amt)
        }
    }
}
