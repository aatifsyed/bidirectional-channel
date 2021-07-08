use async_std::test;
use bidirectional_channel::{bounded, Respond, SendRequestError};
use futures::join;
use ntest::timeout;

#[test]
async fn request_response() {
    let (requester, responder) = bounded(1);
    let requester = async {
        requester
            .send("hello")
            .await
            .expect("Responder or UnRespondedRequest was dropped")
    };
    let responder = async {
        let request = responder.recv().await.expect("Requester was dropped");
        let len = request.len();
        request.respond(len).unwrap()
    };
    let (response, request) = join!(requester, responder);
    assert!(request.len() == response)
}

#[test]
async fn closed() {
    let (requester, responder) = bounded::<_, usize>(1);
    drop(responder);
    assert!(matches!(
        requester.send("hello").await,
        Err(SendRequestError::Closed(_))
    ))
}

#[test]
async fn cancelled() {
    let (requester, responder) = bounded::<_, usize>(1);
    let (result, _) = join!(requester.send("hello"), async {
        drop(responder.recv().await)
    });
    assert!(matches!(result, Err(SendRequestError::Ignored)))
}

#[test]
#[timeout(10)]
#[should_panic]
async fn deadlock() {
    let (requester, responder) = bounded(1);
    let requester = async {
        requester.send("first").await.unwrap();
        requester.send("second").await.unwrap();
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
