use async_std::channel as mpsc;
use futures::channel::oneshot;
use std::fmt::Debug;

pub use mpsc::Receiver;
pub type Response<T> = oneshot::Receiver<T>;
#[must_use = "You must respond to the request"]
pub type Responder<T> = oneshot::Sender<T>;

pub struct IncomingRequest<Req, Resp>(pub Req, pub Responder<Resp>);

impl<Req, Resp> IncomingRequest<Req, Resp>
where
    Resp: Debug,
{
    pub fn respond(self, response: Resp) -> Result<(), Resp> {
        self.1.send(response)
    }
}

pub struct Sender<Req, Resp> {
    outgoing: mpsc::Sender<IncomingRequest<Req, Resp>>,
}

impl<Req, Resp> Sender<Req, Resp> {
    pub async fn send(
        &self,
        request: Req,
    ) -> Result<Response<Resp>, mpsc::SendError<IncomingRequest<Req, Resp>>> {
        let (responder, response) = oneshot::channel();
        self.outgoing
            .send(IncomingRequest(request, responder))
            .await?;
        Ok(response)
    }
}

/// ```
/// # async_std::task::block_on( async {
/// let (sender, receiver) = bidirectional_channel::bounded::<String, usize>(10);
/// let response = sender.send(String::from("hello")).await.unwrap();
///
/// // This side of the channel receives strings, and responds with their length
/// let request = receiver.recv().await.unwrap();
/// let len = request.len();
/// request.respond(len).unwrap();
///
/// assert!(response.await.unwrap() == 5);
/// # })
/// ```
pub fn bounded<Req, Resp>(
    capacity: usize,
) -> (
    Sender<Req, Resp>,
    mpsc::Receiver<IncomingRequest<Req, Resp>>,
) {
    let (sender, receiver) = mpsc::bounded(capacity);
    (Sender { outgoing: sender }, receiver)
}

// Above this line is core functionality
// Below it, ergonomics

use std::ops::{Deref, DerefMut};

impl<Req, Resp> Deref for IncomingRequest<Req, Resp> {
    type Target = Req;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<Req, Resp> DerefMut for IncomingRequest<Req, Resp> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
