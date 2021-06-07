use async_trait::async_trait;

#[async_trait]
pub trait Actor {
    type Receives: Send;
    type Responds: Send;
    async fn new(&mut self, received: Self::Receives) -> Self::Responds;
}
