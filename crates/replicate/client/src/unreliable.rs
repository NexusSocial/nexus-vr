//! Wraps a [`wtransport::Connection`] to turn it into a [`Stream`] and [`Sink`].

use std::pin::Pin;
use std::sync::Arc;
use std::{fmt::Debug, task::Poll};

use bytes::{Bytes, BytesMut};
use eyre::WrapErr;
use futures::{Future, Sink, Stream, TryFutureExt};
use wtransport::{datagram::Datagram, error::ConnectionError};

/// Trait alias for the future used to receive datagrams.
trait RecvFut: Future<Output = Result<Datagram, ConnectionError>> + Send {}
impl<T> RecvFut for T where T: Future<Output = Result<Datagram, ConnectionError>> + Send {}

/// Wraps a [`wtransport::Connection`] to turn it into a [`Stream`] and [`Sink`] of unreliable
/// messages of bytes.
pub(crate) struct BiUnreliable {
	c: Arc<wtransport::Connection>,
	recv_fut: Option<Pin<Box<dyn RecvFut>>>,
}

impl BiUnreliable {
	pub fn new(conn: wtransport::Connection) -> Self {
		Self {
			c: Arc::new(conn),
			recv_fut: None,
		}
	}
}

impl Debug for BiUnreliable {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.c.fmt(f)
	}
}

impl Stream for BiUnreliable {
	type Item = eyre::Result<BytesMut>;

	fn poll_next(
		mut self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> Poll<Option<Self::Item>> {
		if self.recv_fut.is_none() {
			let c_clone = self.c.clone();
			let fut = async move { c_clone.receive_datagram().await };
			self.recv_fut.replace(Box::pin(fut));
		}
		let poll_result = self.recv_fut.as_mut().unwrap().try_poll_unpin(cx);
		poll_result.map(|result| {
			Some(
				result
					// TODO: Remove this clone somehow :(
					.map(|datagram| BytesMut::from(datagram.payload().as_ref()))
					.wrap_err("failed to receive datagram"),
			)
		})
	}
}

impl Sink<Bytes> for BiUnreliable {
	type Error = eyre::Report;

	fn poll_ready(
		self: Pin<&mut Self>,
		_cx: &mut std::task::Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}

	fn start_send(self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
		self.c
			.send_datagram(item)
			.wrap_err("failed to send datagram")
	}

	fn poll_flush(
		self: Pin<&mut Self>,
		_cx: &mut std::task::Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		Poll::Ready(Ok(()))
	}

	fn poll_close(
		self: Pin<&mut Self>,
		_cx: &mut std::task::Context<'_>,
	) -> Poll<Result<(), Self::Error>> {
		self.c.close(Default::default(), b"poll_close");
		Poll::Ready(Ok(()))
	}
}
