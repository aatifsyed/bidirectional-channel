use async_std::channel as mpsc;
use futures::channel::oneshot;
use std::fmt::Debug;

pub type Response<T> = oneshot::Receiver<T>;
#[must_use = "You must respond to the request"] // TODO doesn't work
pub struct Request<Req, Resp> {
    request: Req,
    responder: oneshot::Sender<Resp>,
}

impl<Req, Resp> Request<Req, Resp>
where
    Resp: Debug,
{
    pub fn respond(self, response: Resp) -> Result<(), Resp> {
        self.responder.send(response)
    }
}

pub struct Sender<Req, Resp> {
    outgoing: mpsc::Sender<Request<Req, Resp>>,
}

impl<Req, Resp> Sender<Req, Resp> {
    pub async fn send(
        &self,
        request: Req,
    ) -> Result<Response<Resp>, mpsc::SendError<Request<Req, Resp>>> {
        let (responder, response) = oneshot::channel();
        self.outgoing.send(Request { request, responder }).await?;
        Ok(response)
    }
}

/// ```
/// # async_std::task::block_on( async {
/// let (sender, receiver) = aactor::bounded::<String, usize>(10);
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
) -> (Sender<Req, Resp>, mpsc::Receiver<Request<Req, Resp>>) {
    let (sender, receiver) = mpsc::bounded(capacity);
    (Sender { outgoing: sender }, receiver)
}

// Above this line is core functionality
// Below it, ergonomics

use std::ops::{Deref, DerefMut};

impl<Req, Resp> Deref for Request<Req, Resp> {
    type Target = Req;
    fn deref(&self) -> &Self::Target {
        &self.request
    }
}
impl<Req, Resp> DerefMut for Request<Req, Resp> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.request
    }
}
