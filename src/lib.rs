//! An async channel with request-response semantics
//! See the [`bounded`] function documentation for more

#[deny(missing_docs)]
use async_std::channel;
pub use async_std::channel::Receiver as Responder;
use derive_more::{AsMut, AsRef, From};
use futures::channel::oneshot;
pub type FutureResponse<T> = oneshot::Receiver<T>;

pub trait Respond<Resp> {
    /// If the implementer owns any data, it is given back to the user on both receipt and failure
    type Owned;
    /// Fullfill our obligation to the [`Requester`] by responding to their request
    fn respond(self, response: Resp) -> Result<Self::Owned, (Self::Owned, Resp)>;
}

mod impl_respond {
    use super::{ReceivedRequest, Respond, UnRespondedRequest};

    impl<Req, Resp> Respond<Resp> for ReceivedRequest<Req, Resp> {
        type Owned = Req;
        fn respond(self, response: Resp) -> Result<Self::Owned, (Self::Owned, Resp)> {
            match self.responder.respond(response) {
                Ok(_) => Ok(self.request),
                Err((_, response)) => Err((self.request, response)),
            }
        }
    }

    impl<Resp> Respond<Resp> for UnRespondedRequest<Resp> {
        type Owned = ();
        fn respond(self, response: Resp) -> Result<Self::Owned, (Self::Owned, Resp)> {
            self.responder
                .send(response)
                .map_err(|response| ((), response))
        }
    }

    impl<Req, Resp> Respond<Resp> for (Req, UnRespondedRequest<Resp>) {
        type Owned = Req;
        fn respond(self, response: Resp) -> Result<Self::Owned, (Self::Owned, Resp)> {
            match self.1.respond(response) {
                Ok(_) => Ok(self.0),
                Err((_, response)) => Err((self.0, response)),
            }
        }
    }
}

/// Represents that the [`Requester`] associated with this communication is still waiting for a response.
/// Must be used by calling [`Respond::respond`].
#[must_use = "You must respond to the request"]
pub struct UnRespondedRequest<Resp> {
    responder: oneshot::Sender<Resp>,
}

/// Represents the request.
/// This implements [`AsRef`] and [`AsMut`] for the request itself.
/// May also be destructured.
/// Must be used by calling [`Respond::respond`].
#[must_use = "You must respond to the request"]
#[derive(AsRef, AsMut, From)]
pub struct ReceivedRequest<Req, Resp> {
    #[as_ref]
    #[as_mut]
    pub request: Req,
    pub responder: UnRespondedRequest<Resp>,
}

/// Represents the initiator for the request-response exchange
pub struct Requester<Req, Resp> {
    outgoing: channel::Sender<(Req, UnRespondedRequest<Resp>)>,
}

impl<Req, Resp> Requester<Req, Resp> {
    /// Make a request.
    /// The [`FutureResponse`] should be `await`ed to get the response from the responder
    pub async fn send(&self, request: Req) -> Result<FutureResponse<Resp>, Req> {
        // Create the return path
        let (response_sender, response_receiver) = oneshot::channel();
        self.outgoing
            .send((
                request,
                UnRespondedRequest {
                    responder: response_sender,
                },
            ))
            .await
            .map_err(|e| e.into_inner().0)?;
        Ok(response_receiver)
    }
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Create a bounded [`Requester`]-[`Responder`] pair.  
/// That is, once the channel is full, future senders will yield when awaiting until there's space again
///
/// Terminology is as follows:
/// ```mermaid
/// sequenceDiagram
///     Requester ->> Responder: request
///     Responder ->> Requester: response
/// ```
///
/// When a [`Responder`] is asked to receive a request, it returns a `(Request, [UnRespondedRequest<Response>])` pair.
/// The latter should be used to communicate back to the sender
///
/// ```
/// # async_std::task::block_on( async {
/// let (requester, responder) = bidirectional_channel::bounded::<String, usize>(10);
/// let response = requester.send(String::from("hello")).await.unwrap();
///
/// // This side of the channel receives strings, and responds with their length
/// let (request, unfulfilled_response) = responder.recv().await.unwrap();
/// let len = request.len();
/// unfulfilled_response.respond(len).unwrap();
///
/// assert!(response.await.unwrap() == 5);
/// # })
/// ```
pub fn bounded<Req, Resp>(
    capacity: usize,
) -> (Requester<Req, Resp>, Responder<ReceivedRequest<Req, Resp>>) {
    let (sender, receiver) = channel::bounded(capacity);
    (Requester { outgoing: sender }, receiver)
}
