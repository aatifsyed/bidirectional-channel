<h1 align="center">bidirectional-channel</h1>
<div align="center">
 <strong>
   Async channel with request-response semantics
 </strong>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/bidirectional-channel">
    <img src="https://img.shields.io/crates/v/bidirectional-channel.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/bidirectional-channel">
    <img src="https://img.shields.io/crates/d/bidirectional-channel.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/bidirectional-channel">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

```rust
use bidirectional_channel::{bounded};
use futures::join;

let (requester, responder) = bounded(1);

// A requesting task
let requester = async {
    requester
        .send("hello")
        .await
        .expect("Responder or UnRespondedRequest was dropped")
};

// A responding task.
// This receives an &str, and returns its length
let responder = async {
    let request = responder.recv().await.expect("Requester was dropped");
    let len = request.len();
    request.respond(len).unwrap()
};

// Perform the exchange
let (response, request) = join!(requester, responder);

assert!(request.len() == response)
```
