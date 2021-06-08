use aactor::channel::{bounded, Receiver, Request};
use async_std::test;
use futures::{join, stream, StreamExt};
use log::debug;
use pretty_env_logger;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy)]
struct Ping;

struct Counter {
    count: AtomicUsize,
    receiver: Receiver<Request<Ping, ()>>,
}
impl Counter {
    async fn main(&mut self) {
        while let Some(request) = self.receiver.next().await {
            debug!("received a ping");
            self.count.fetch_add(1, Ordering::SeqCst);
            request.respond(()).unwrap();
        }
    }
}

#[test]
async fn counter() {
    pretty_env_logger::init();
    let (sender, receiver) = bounded(1);
    let mut counter = Counter {
        count: AtomicUsize::new(0),
        receiver,
    };
    let send_task = async {
        let mut stream = stream::repeat(Ping).take(5);
        while let Some(ping) = stream.next().await {
            let response = sender.send(ping).await.unwrap();
            debug!("sent a ping");
            response.await.unwrap();
        }
        drop(sender) // End the test
    };
    let receive_task = counter.main();
    join!(send_task, receive_task);
    assert!(counter.count.load(Ordering::SeqCst) == 5);
}
