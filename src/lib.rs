//! A standard async channel, except the receiver can respond to each message, and the sender can await for that response.
//! See the [`bounded`] function documentation for more

#[deny(missing_docs)]
use async_std::channel;
pub use async_std::channel::Receiver as Responder;
use futures::channel::oneshot;
use std::fmt::Debug;
pub type FutureResponse<T> = oneshot::Receiver<T>;

/// Received by the [`Responder`] - represents that the [`Requester`] associated with this channel is still waiting for a response.
/// Must be used.
#[must_use = "You must respond to the request"]
pub struct UnRespondedRequest<Resp> {
    responder: oneshot::Sender<Resp>,
}

impl<Resp> UnRespondedRequest<Resp>
where
    Resp: Debug,
{
    /// Fullfill our obligation to the [`Requester`].
    pub fn respond(self, response: Resp) -> Result<(), Resp> {
        self.responder.send(response)
    }
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
) -> (
    Requester<Req, Resp>,
    Responder<(Req, UnRespondedRequest<Resp>)>,
) {
    let (sender, receiver) = channel::bounded(capacity);
    (Requester { outgoing: sender }, receiver)
}
