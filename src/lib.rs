#![deny(missing_docs)]
//! An async channel with request-response semantics.  
//! When a [`Responder`] is asked to receive a request, it returns a [`ReceivedRequest`],
//! which should be used to communicate back to the sender
//!
//! ```
//! # async_std::task::block_on( async {
//! use bidirectional_channel::Respond; // Don't forget to import this trait
//! let (requester, responder) = bidirectional_channel::bounded(10);
//! let future_response = requester.send(String::from("hello")).await.unwrap();
//!
//! // This side of the channel receives Strings, and responds with their length
//! let received_request = responder.recv().await.unwrap();
//! let len = received_request.len(); // Deref coercion to the actual request
//! received_request.respond(len).unwrap();
//!
//! assert!(future_response.await.unwrap() == 5);
//! # })
//! ```

use async_std::channel;
pub use async_std::channel::Receiver as Responder;
use derive_more::{AsMut, AsRef, Deref, DerefMut, From};
use futures::channel::oneshot;
#[allow(unused_imports)]
use std::ops::{Deref, DerefMut}; // Doc links

/// Represents waiting for the [`Responder`] to [`Respond::respond`].
pub type FutureResponse<T> = oneshot::Receiver<T>;

/// Represents fullfilling a [`Requester`]'s request.
/// This trait is sealed - types external to this crate may not implement it.
pub trait Respond<Resp>: impl_respond::Sealed {
    /// If the implementer owns any data, it is given back to the user on both receipt and failure
    type Owned;
    /// Fullfill our obligation to the [`Requester`] by responding to their request
    fn respond(self, response: Resp) -> Result<Self::Owned, (Self::Owned, Resp)>;
}

mod impl_respond {
    pub trait Sealed {}
    impl<Req, Resp> Sealed for ReceivedRequest<Req, Resp> {}
    impl<Resp> Sealed for UnRespondedRequest<Resp> {}

    use super::{ReceivedRequest, Respond, UnRespondedRequest};

    impl<Req, Resp> Respond<Resp> for ReceivedRequest<Req, Resp> {
        type Owned = Req;
        fn respond(self, response: Resp) -> Result<Self::Owned, (Self::Owned, Resp)> {
            match self.unresponded.respond(response) {
                Ok(_) => Ok(self.request),
                Err((_, response)) => Err((self.request, response)),
            }
        }
    }

    impl<Resp> Respond<Resp> for UnRespondedRequest<Resp> {
        type Owned = ();
        fn respond(self, response: Resp) -> Result<Self::Owned, (Self::Owned, Resp)> {
            self.response_sender
                .send(response)
                .map_err(|response| ((), response))
        }
    }
}

/// Represents that the [`Requester`] associated with this communication is still waiting for a response.
/// Must be used by calling [`Respond::respond`].
#[must_use = "You must respond to the request"]
pub struct UnRespondedRequest<Resp> {
    response_sender: oneshot::Sender<Resp>,
}

/// Represents the request.
/// This implements [`AsRef`] and [`AsMut`] for the request itself for explicit use.
/// Alternatively, you may use [`Deref`] and [`DerefMut`] either explicitly, or coerced.
/// Must be used by calling [`Respond::respond`], or destructured.
#[must_use = "You must respond to the request"]
#[derive(AsRef, AsMut, From, Deref, DerefMut)]
pub struct ReceivedRequest<Req, Resp> {
    /// The request itself
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    pub request: Req,
    /// Handle to [`Respond::respond`] to the [`Requester`]
    pub unresponded: UnRespondedRequest<Resp>,
}

/// Represents the initiator for the request-response exchange
pub struct Requester<Req, Resp> {
    outgoing: channel::Sender<ReceivedRequest<Req, Resp>>,
}

impl<Req, Resp> Requester<Req, Resp> {
    /// Make a request.
    /// The [`FutureResponse`] should be `await`ed to get the response from the responder
    pub async fn send(&self, request: Req) -> Result<FutureResponse<Resp>, Req> {
        // Create the return path
        let (response_sender, response_receiver) = oneshot::channel();
        self.outgoing
            .send(ReceivedRequest {
                request,
                unresponded: UnRespondedRequest { response_sender },
            })
            .await
            .map_err(|e| e.into_inner().request)?;
        Ok(response_receiver)
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
