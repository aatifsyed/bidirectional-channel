mod actor;
mod bidirectional_channel;

pub use bidirectional_channel::{new, Receiver, Sender};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
