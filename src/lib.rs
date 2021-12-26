#![deny(missing_docs)]
#![deny(unused)]
//! An async channel with request-response semantics.  
//! When a [`Responder`] is asked to receive a request, it returns a [`ReceivedRequest`],
//! which should be used to communicate back to the sender
//!
//! ```
//! # use futures::join;
//! # async_std::task::block_on( async {
//! use bidirectional_channel::{bounded};
//! let (requester, responder) = bounded(1);
//! let requester = async { requester.send("hello").await.unwrap() };
//! let responder = async {
//!     let request = responder.recv().await.unwrap();
//!     let len = request.len();
//!     request.respond(len).unwrap()
//! };
//! let (response, request) = join!(requester, responder);
//! assert!(request.len() == response)
//! # })
//! ```

use async_std::channel;
/// An [`async_std::channel::Receiver`] which receives an [`UnRespondedRequest<Req, Resp>`] instead of a `Req`.
pub use async_std::channel::Receiver as Responder;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use futures::channel::oneshot;
use std::fmt::Debug;
#[cfg(doc)]
use std::ops::{Deref, DerefMut};
use thiserror::Error;

/// Error returned when sending a request
#[derive(Error)]
pub enum SendRequestError<Req> {
    /// The [`Responder`] for this channel was dropped.
    /// Returns ownership of the `Req` that failed to send
    #[error("The Responder was dropped before the message was sent")]
    Closed(Req),
    /// The [`UnRespondedRequest`] for this request was dropped.
    #[error("The UnRespondedRequest was dropped, not responded to")]
    Ignored,
}
impl<Req> Debug for SendRequestError<Req> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Closed(_) => write!(f, "Closed(..)"),
            Self::Ignored => write!(f, "Cancelled"),
        }
    }
}

/// Represents that the [`Requester`] associated with this communication is still waiting for a response.
#[must_use = "You must respond to the request"]
pub struct UnRespondedRequest<Resp> {
    response_sender: oneshot::Sender<Resp>,
}
impl<Resp> UnRespondedRequest<Resp> {
    /// Respond to the [`Requester`]'s request.
    /// Fails if the associated [`Requester`] was dropped, and returns your response back
    pub fn respond(self, response: Resp) -> Result<(), Resp> {
        self.response_sender.send(response)
    }
}

/// Represents the request.
/// This implements [`AsRef`] and [`AsMut`] for the request itself for explicit use.
/// Alternatively, you may use [`Deref`] and [`DerefMut`] either explicitly, or coerced.
/// Must be used by calling [`ReceivedRequest::respond`], or destructured.
#[must_use = "You must respond to the request"]
#[derive(AsRef, AsMut, Deref, DerefMut)]
pub struct ReceivedRequest<Req, Resp> {
    /// The request itself
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    pub request: Req,
    /// Handle to respond to the [`Requester`]
    pub unresponded: UnRespondedRequest<Resp>,
}

impl<Req, Resp> ReceivedRequest<Req, Resp> {
    /// Respond to the [`Requester`]'s request, and take ownership of it
    /// Fails if the associated [`Requester`] was dropped, and returns your response back
    pub fn respond(self, response: Resp) -> Result<Req, (Req, Resp)> {
        match self.unresponded.respond(response) {
            Ok(_) => Ok(self.request),
            Err(response) => Err((self.request, response)),
        }
    }
}

impl<Req, Resp> Into<(Req, UnRespondedRequest<Resp>)> for ReceivedRequest<Req, Resp> {
    fn into(self) -> (Req, UnRespondedRequest<Resp>) {
        let ReceivedRequest {
            request,
            unresponded,
        } = self;
        (request, unresponded)
    }
}
/// Represents the initiator for the request-response exchange
#[derive(Clone)]
pub struct Requester<Req, Resp> {
    outgoing: channel::Sender<ReceivedRequest<Req, Resp>>,
}

impl<Req, Resp> Requester<Req, Resp> {
    /// Make a request.
    /// `await` the result to receive the response.
    pub async fn send(&self, request: Req) -> Result<Resp, SendRequestError<Req>> {
        // Create the return path
        let (response_sender, response_receiver) = oneshot::channel();
        self.outgoing
            .send(ReceivedRequest {
                request,
                unresponded: UnRespondedRequest { response_sender },
            })
            .await
            .map_err(|e| SendRequestError::Closed(e.into_inner().request))?;
        let response = response_receiver
            .await
            .map_err(|_| SendRequestError::Ignored)?;
        Ok(response)
    }
}

/// Create a bounded [`Requester`]-[`Responder`] pair.  
/// That is, once the channel is full, future senders will yield when awaiting until there's space again
pub fn bounded<Req, Resp>(
    capacity: usize,
) -> (Requester<Req, Resp>, Responder<ReceivedRequest<Req, Resp>>) {
    let (sender, receiver) = channel::bounded(capacity);
    (Requester { outgoing: sender }, receiver)
}

/// Create an ubounded [`Requester`]-[`Responder`] pair.  
pub fn unbounded<Req, Resp>() -> (Requester<Req, Resp>, Responder<ReceivedRequest<Req, Resp>>) {
    let (sender, receiver) = channel::unbounded();
    (Requester { outgoing: sender }, receiver)
}
