//! Boilerplate to wrap tokio_serde::Framed.
//!
//! Improves errors and makes things a tiny bit easier to use.
//! Also gives us the ability to implement traits such as Debug.

use std::pin::pin;

use futures::{Sink, Stream};
use pin_project::pin_project;
use tokio_util::codec::LengthDelimitedCodec;

/// Frames `Transport`, converting a byte-oriented `AsyncRead + AsyncWrite` into a
/// framed/message-oriented `Stream + Sink` of `Item` and `ItemSink`.
///
/// Framing is a way of converting from raw bytes into messages/frames. For example, prefixing
/// every message by its length is a simple way of converting a byte-oriented TCP stream into a
/// message-oriented stream.
///
/// Replicate uses length prefixes on messages to do the basic framing, as well as
/// serde_json to do the serialization (for now). Future versions will explicitly use
/// protobufs or capnproto to do the serialization and try to be zero-copy.
#[pin_project]
pub struct Framed<Transport, Item, ItemSink> {
	#[pin]
	inner: tokio_serde::Framed<
		tokio_util::codec::Framed<Transport, LengthDelimitedCodec>,
		Item,
		ItemSink,
		tokio_serde::formats::Json<Item, ItemSink>,
	>,
}

impl<Transport, Item, ItemSink> Framed<Transport, Item, ItemSink>
where
	Transport: tokio::io::AsyncWrite + tokio::io::AsyncRead,
{
	/// Frames `transport`, converting a byte-oriented `AsyncRead + AsyncWrite` into a
	/// framed/message-oriented `Stream + Sink` of `Item` and `ItemSink`.
	pub fn new(transport: Transport) -> Self {
		let framed =
			tokio_util::codec::Framed::new(transport, LengthDelimitedCodec::new());
		let json_codec: tokio_serde::formats::Json<Item, ItemSink> = Default::default();
		let framed = tokio_serde::Framed::new(framed, json_codec);
		Self { inner: framed }
	}
}

// Skip potentially !Debug transport.
impl<Transport, Item, ItemSink> std::fmt::Debug for Framed<Transport, Item, ItemSink> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct(std::any::type_name::<Self>()).finish()
	}
}

// -- boilerplate to implement stream and sink by calling into the inner type --

impl<Transport, Item, ItemSink> Stream for Framed<Transport, Item, ItemSink>
where
	Transport: tokio::io::AsyncWrite + tokio::io::AsyncRead,
	Item: for<'a> serde::Deserialize<'a>,
{
	type Item = std::io::Result<Item>;

	fn poll_next(
		self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Option<Self::Item>> {
		self.project().inner.poll_next(cx)
	}
}

impl<Transport, Item, ItemSink> Sink<ItemSink> for Framed<Transport, Item, ItemSink>
where
	Transport: tokio::io::AsyncWrite + tokio::io::AsyncRead,
	ItemSink: serde::Serialize,
{
	type Error = std::io::Error;

	fn poll_ready(
		self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<std::result::Result<(), Self::Error>> {
		self.project().inner.poll_ready(cx)
	}

	fn start_send(
		self: std::pin::Pin<&mut Self>,
		item: ItemSink,
	) -> std::result::Result<(), Self::Error> {
		self.project().inner.start_send(item)
	}

	fn poll_flush(
		self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<std::result::Result<(), Self::Error>> {
		self.project().inner.poll_flush(cx)
	}

	fn poll_close(
		self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<std::result::Result<(), Self::Error>> {
		self.project().inner.poll_close(cx)
	}
}
