// SPDX-License-Identifier: Apache-2.0 OR MIT

use core::{
    future::Future,
    ops,
    panic::Location,
    pin::Pin,
    task::{Context, Poll},
};
use std::thread;

use pin_project_lite::pin_project;

pin_project! {
    /// A type that asserts that the underlying type is not moved after being pinned
    /// and mutably accessed.
    ///
    /// See the [crate-level documentation](crate) for details.
    #[project(!Unpin)]
    #[derive(Debug)]
    pub struct AssertUnmoved<T> {
        #[pin]
        inner: T,
        this_addr: usize,
        first_pinned_mutably_accessed_at: Option<&'static Location<'static>>,
    }
    impl<T> PinnedDrop for AssertUnmoved<T> {
        /// # Panics
        ///
        /// Panics if this `AssertUnmoved` moved after being pinned and mutably accessed.
        fn drop(this: Pin<&mut Self>) {
            // If the thread is panicking then we can't panic again as that will
            // cause the process to be aborted.
            if !thread::panicking() && this.this_addr != 0 {
                let cur_this = this.addr();
                assert_eq!(
                    this.this_addr,
                    cur_this,
                    "AssertUnmoved moved before drop\n\
                     \tfirst pinned mutably accessed at {}\n",
                    this.first_pinned_mutably_accessed_at.unwrap()
                );
            }
        }
    }
}

impl<T> AssertUnmoved<T> {
    /// Creates a new `AssertUnmoved`.
    #[must_use]
    pub const fn new(inner: T) -> Self {
        Self { inner, this_addr: 0, first_pinned_mutably_accessed_at: None }
    }

    /// Gets a reference to the underlying type.
    ///
    /// Unlike [`get_mut`](AssertUnmoved::get_mut) method, this method can always called.
    ///
    /// You can also access the underlying type via [`Deref`](std::ops::Deref) impl.
    #[must_use]
    pub const fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the underlying type.
    ///
    /// Note that this method can only be called before pinned since
    /// `AssertUnmoved` is `!Unpin` (this is guaranteed by the type system!).
    ///
    /// # Panics
    ///
    /// Panics if this `AssertUnmoved` moved after being pinned and mutably accessed.
    #[must_use]
    #[track_caller]
    pub fn get_mut(&mut self) -> &mut T {
        if self.this_addr != 0 {
            let cur_this = self.addr();
            assert_eq!(
                self.this_addr,
                cur_this,
                "AssertUnmoved moved after get_pin_mut call\n\
                 \tfirst pinned mutably accessed at {}\n",
                self.first_pinned_mutably_accessed_at.unwrap()
            );
        }
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
    /// ```
    /// use std::{
    ///     pin::Pin,
    ///     task::{Context, Poll},
    /// };
    ///
    /// use assert_unmoved::AssertUnmoved;
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
    /// [`Stream`]: https://docs.rs/futures/latest/futures/stream/trait.Stream.html
    #[must_use]
    #[track_caller]
    pub fn get_pin_mut(mut self: Pin<&mut Self>) -> Pin<&mut T> {
        let cur_this = self.addr();
        if self.this_addr == 0 {
            // First time being pinned and mutably accessed.
            *self.as_mut().project().this_addr = cur_this;
            *self.as_mut().project().first_pinned_mutably_accessed_at = Some(Location::caller());
        } else {
            assert_eq!(
                self.this_addr,
                cur_this,
                "AssertUnmoved moved between get_pin_mut calls\n\
                 \tfirst pinned mutably accessed at {}\n",
                self.first_pinned_mutably_accessed_at.unwrap()
            );
        }
        self.project().inner
    }

    fn addr(&self) -> usize {
        self as *const Self as usize
    }
}

impl<T> ops::Deref for AssertUnmoved<T> {
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

    #[track_caller]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_pin_mut().poll(cx)
    }
}

#[cfg(feature = "futures03")]
#[cfg_attr(docsrs, doc(cfg(feature = "futures03")))]
mod futures03 {
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };

    use futures_core::{
        future::FusedFuture,
        stream::{FusedStream, Stream},
    };
    use futures_io as io;
    use futures_sink::Sink;

    use super::AssertUnmoved;

    impl<F: FusedFuture> FusedFuture for AssertUnmoved<F> {
        fn is_terminated(&self) -> bool {
            self.get_ref().is_terminated()
        }
    }

    impl<S: Stream> Stream for AssertUnmoved<S> {
        type Item = S::Item;

        #[track_caller]
        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.get_pin_mut().poll_next(cx)
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.get_ref().size_hint()
        }
    }

    impl<S: FusedStream> FusedStream for AssertUnmoved<S> {
        fn is_terminated(&self) -> bool {
            self.get_ref().is_terminated()
        }
    }

    impl<S: Sink<Item>, Item> Sink<Item> for AssertUnmoved<S> {
        type Error = S::Error;

        #[track_caller]
        fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.get_pin_mut().poll_ready(cx)
        }

        #[track_caller]
        fn start_send(self: Pin<&mut Self>, item: Item) -> Result<(), Self::Error> {
            self.get_pin_mut().start_send(item)
        }

        #[track_caller]
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.get_pin_mut().poll_flush(cx)
        }

        #[track_caller]
        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.get_pin_mut().poll_close(cx)
        }
    }

    impl<R: io::AsyncRead> io::AsyncRead for AssertUnmoved<R> {
        #[track_caller]
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_read(cx, buf)
        }

        #[track_caller]
        fn poll_read_vectored(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &mut [io::IoSliceMut<'_>],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_read_vectored(cx, bufs)
        }
    }

    impl<W: io::AsyncWrite> io::AsyncWrite for AssertUnmoved<W> {
        #[track_caller]
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_write(cx, buf)
        }

        #[track_caller]
        fn poll_write_vectored(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[io::IoSlice<'_>],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_write_vectored(cx, bufs)
        }

        #[track_caller]
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_flush(cx)
        }

        #[track_caller]
        fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_close(cx)
        }
    }

    impl<S: io::AsyncSeek> io::AsyncSeek for AssertUnmoved<S> {
        #[track_caller]
        fn poll_seek(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            pos: io::SeekFrom,
        ) -> Poll<io::Result<u64>> {
            self.get_pin_mut().poll_seek(cx, pos)
        }
    }

    impl<R: io::AsyncBufRead> io::AsyncBufRead for AssertUnmoved<R> {
        #[track_caller]
        fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
            self.get_pin_mut().poll_fill_buf(cx)
        }

        #[track_caller]
        fn consume(self: Pin<&mut Self>, amt: usize) {
            self.get_pin_mut().consume(amt);
        }
    }
}

#[cfg(feature = "tokio02")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio02")))]
mod tokio02 {
    use core::{
        mem::MaybeUninit,
        pin::Pin,
        task::{Context, Poll},
    };
    use std::io;

    use bytes05::{Buf, BufMut};
    use tokio02_crate::io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite};

    use super::AssertUnmoved;

    impl<R: AsyncRead> AsyncRead for AssertUnmoved<R> {
        unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [MaybeUninit<u8>]) -> bool {
            // SAFETY: The safety contract must be upheld by the caller.
            unsafe { self.get_ref().prepare_uninitialized_buffer(buf) }
        }

        #[track_caller]
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_read(cx, buf)
        }

        #[track_caller]
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
        #[track_caller]
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_write(cx, buf)
        }

        #[track_caller]
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_flush(cx)
        }

        #[track_caller]
        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_shutdown(cx)
        }

        #[track_caller]
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
        #[track_caller]
        fn start_seek(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            pos: io::SeekFrom,
        ) -> Poll<io::Result<()>> {
            self.get_pin_mut().start_seek(cx, pos)
        }

        #[track_caller]
        fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
            self.get_pin_mut().poll_complete(cx)
        }
    }

    impl<R: AsyncBufRead> AsyncBufRead for AssertUnmoved<R> {
        #[track_caller]
        fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
            self.get_pin_mut().poll_fill_buf(cx)
        }

        #[track_caller]
        fn consume(self: Pin<&mut Self>, amt: usize) {
            self.get_pin_mut().consume(amt);
        }
    }
}

#[cfg(feature = "tokio03")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio03")))]
mod tokio03 {
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };

    use tokio03_crate::io;

    use super::AssertUnmoved;

    impl<R: io::AsyncRead> io::AsyncRead for AssertUnmoved<R> {
        #[track_caller]
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut io::ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_read(cx, buf)
        }
    }

    impl<W: io::AsyncWrite> io::AsyncWrite for AssertUnmoved<W> {
        #[track_caller]
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_write(cx, buf)
        }

        #[track_caller]
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_flush(cx)
        }

        #[track_caller]
        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_shutdown(cx)
        }
    }

    impl<S: io::AsyncSeek> io::AsyncSeek for AssertUnmoved<S> {
        #[track_caller]
        fn start_seek(self: Pin<&mut Self>, pos: io::SeekFrom) -> io::Result<()> {
            self.get_pin_mut().start_seek(pos)
        }

        #[track_caller]
        fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
            self.get_pin_mut().poll_complete(cx)
        }
    }

    impl<R: io::AsyncBufRead> io::AsyncBufRead for AssertUnmoved<R> {
        #[track_caller]
        fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
            self.get_pin_mut().poll_fill_buf(cx)
        }

        #[track_caller]
        fn consume(self: Pin<&mut Self>, amt: usize) {
            self.get_pin_mut().consume(amt);
        }
    }
}

#[cfg(feature = "tokio1")]
#[cfg_attr(docsrs, doc(cfg(feature = "tokio1")))]
mod tokio1 {
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };

    use tokio1_crate::io;

    use super::AssertUnmoved;

    impl<R: io::AsyncRead> io::AsyncRead for AssertUnmoved<R> {
        #[track_caller]
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut io::ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_read(cx, buf)
        }
    }

    impl<W: io::AsyncWrite> io::AsyncWrite for AssertUnmoved<W> {
        #[track_caller]
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<io::Result<usize>> {
            self.get_pin_mut().poll_write(cx, buf)
        }

        #[track_caller]
        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_flush(cx)
        }

        #[track_caller]
        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
            self.get_pin_mut().poll_shutdown(cx)
        }

        #[track_caller]
        fn poll_write_vectored(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> Poll<Result<usize, io::Error>> {
            self.get_pin_mut().poll_write_vectored(cx, bufs)
        }

        fn is_write_vectored(&self) -> bool {
            self.get_ref().is_write_vectored()
        }
    }

    impl<S: io::AsyncSeek> io::AsyncSeek for AssertUnmoved<S> {
        #[track_caller]
        fn start_seek(self: Pin<&mut Self>, pos: io::SeekFrom) -> io::Result<()> {
            self.get_pin_mut().start_seek(pos)
        }

        #[track_caller]
        fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<u64>> {
            self.get_pin_mut().poll_complete(cx)
        }
    }

    impl<R: io::AsyncBufRead> io::AsyncBufRead for AssertUnmoved<R> {
        #[track_caller]
        fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
            self.get_pin_mut().poll_fill_buf(cx)
        }

        #[track_caller]
        fn consume(self: Pin<&mut Self>, amt: usize) {
            self.get_pin_mut().consume(amt);
        }
    }
}
