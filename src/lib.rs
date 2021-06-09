//! A standard async channel, except the receiver can respond to each message, and the sender can await for that response.
//! See the [`bounded`] function documentation for more

#[deny(missing_docs)]
use async_std::channel;
pub use async_std::channel::Receiver as Responder;
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use futures::channel::oneshot;
use std::fmt::Debug;
pub type FutureResponse<T> = oneshot::Receiver<T>;

#[must_use = "You must respond to the request"]
#[derive(AsRef, AsMut, Deref, DerefMut)]
pub struct IncomingRequest<Req, Resp> {
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    request: Req,
    responder: oneshot::Sender<Resp>,
}

#[must_use = "You must respond to the request"]
pub struct UnfulfilledRequest<Resp> {
    responder: oneshot::Sender<Resp>,
}

impl<Req, Resp> IncomingRequest<Req, Resp>
where
    Resp: Debug,
{
    pub fn respond(self, response: Resp) -> Result<(), Resp> {
        self.responder.send(response)
    }
}

impl<Req, Resp> From<(Req, UnfulfilledRequest<Resp>)> for IncomingRequest<Req, Resp> {
    fn from((request, unfulfilled_request): (Req, UnfulfilledRequest<Resp>)) -> Self {
        Self {
            request,
            responder: unfulfilled_request.responder,
        }
    }
}

pub struct Requester<Req, Resp> {
    outgoing: channel::Sender<IncomingRequest<Req, Resp>>,
}

impl<Req, Resp> Requester<Req, Resp> {
    pub async fn send(
        &self,
        request: Req,
    ) -> Result<FutureResponse<Resp>, channel::SendError<IncomingRequest<Req, Resp>>> {
        let (response_sender, response_receiver) = oneshot::channel();
        self.outgoing
            .send(IncomingRequest {
                request,
                responder: response_sender,
            })
            .await?;
        Ok(response_receiver)
    }
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Create a bounded [`Requester`]-[`Responder`] pair.  
/// Once the channel is full, future senders will yield when awaiting
///
/// Terminology is as follows:
/// ```mermaid
/// sequenceDiagram
///     Requester ->> Responder: request
///     Responder ->> Requester: response
/// ```
///
/// When a [`Responder`] is asked to receive a request, it returns an [`IncomingRequest`].
/// This may be dereferenced to the underlying request
///
/// ```
/// # async_std::task::block_on( async {
/// let (requester, responder) = bidirectional_channel::bounded::<String, usize>(10);
/// let response = requester.send(String::from("hello")).await.unwrap();
///
/// // This side of the channel receives strings, and responds with their length
/// let request = responder.recv().await.unwrap();
/// let len = request.len();
/// request.respond(len).unwrap();
///
/// assert!(response.await.unwrap() == 5);
/// # })
/// ```
pub fn bounded<Req, Resp>(
    capacity: usize,
) -> (Requester<Req, Resp>, Responder<IncomingRequest<Req, Resp>>) {
    let (sender, receiver) = channel::bounded(capacity);
    (Requester { outgoing: sender }, receiver)
}
