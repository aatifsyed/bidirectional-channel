use bidirectional_channel::bounded;
use futures::join;
use tokio::test;

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
