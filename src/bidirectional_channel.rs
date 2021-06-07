use futures::channel::mpsc::{
    channel as unidirectional_channel, Receiver as UnidirectionalReceiver,
    Sender as UnidirectionalSender,
};
pub struct Sender<Outgoing, Incoming> {
    us: UnidirectionalSender<Outgoing>,
    them: UnidirectionalReceiver<Incoming>,
}
impl<Outgoing, Incoming> Sender<Outgoing, Incoming> {
    async fn send(&mut self, outgoing: Outgoing) -> Incoming {
        self.us.
    }
}

pub struct Receiver<Incoming, Outgoing> {
    us: UnidirectionalReceiver<Incoming>,
    them: UnidirectionalSender<Outgoing>,
}

pub fn new<Request, Response>(
    capacity: usize,
) -> (Sender<Request, Response>, Receiver<Request, Response>) {
    let (request_sender, request_receiver) = unidirectional_channel::<Request>(capacity);
    let (response_sender, response_receiver) = unidirectional_channel::<Response>(capacity);
    return (
        Sender {
            us: request_sender,
            them: response_receiver,
        },
        Receiver {
            us: request_receiver,
            them: response_sender,
        },
    );
}
