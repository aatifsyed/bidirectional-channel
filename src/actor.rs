use async_trait::async_trait;

#[async_trait]
pub trait Actor {
    type Receives;
    type Responds;
    async fn recv(&self, received: Self::Receives) -> Self::Responds;
}
