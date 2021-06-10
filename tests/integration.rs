use async_std::test;
use bidirectional_channel::{bounded, Respond};
use futures::join;
use ntest::timeout;

#[test]
async fn request_response() {
    let (requester, responder) = bounded(1);
    let requester = async { requester.send("hello").await.unwrap().await.unwrap() };
    let responder = async {
        let request = responder.recv().await.unwrap();
        let len = request.len();
        request.respond(len).unwrap()
    };
    let (response, request) = join!(requester, responder);
    assert!(request.len() == response)
}

#[test]
#[timeout(10)]
#[should_panic]
async fn deadlock() {
    let (requester, responder) = bounded(1);
    let requester = async {
        requester.send("first").await.unwrap().await.unwrap();
        requester.send("second").await.unwrap().await.unwrap();
    };
    let responder = async {
        let first = responder.recv().await.unwrap();
        let second = responder.recv().await.unwrap();
        let len = first.len();
        first.respond(len).unwrap();
        let len = second.len();
        second.respond(len).unwrap();
    };
    join!(requester, responder);
}
