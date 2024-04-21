#![deny(missing_docs)]
//! API to develop a frontend interface for the afrim.
//!

mod console;
mod message;

pub use afrim_translator::Predicate;
use anyhow::Result;
pub use console::Console;
pub use message::Command;
use std::sync::mpsc::{Receiver, Sender};

/// Trait that every afrim frontend should implement.
///
/// Note that:
/// - the backend can send multiple command at once.
/// - the frontend should send only one command at once.
pub trait Frontend {
    /// Initialize the frontend for the communication.
    fn init(&mut self, _tx: Sender<Command>, _rx: Receiver<Command>) -> Result<()>;
    /// Starts listening for commands.
    fn listen(&mut self) -> Result<()>;
}

/// This frontend do nothing.
pub struct None;

impl Frontend for None {
    fn init(&mut self, _tx: Sender<Command>, _rx: Receiver<Command>) -> Result<()> {
        Ok(())
    }
    fn listen(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::Frontend;
    use std::sync::mpsc;

    #[test]
    fn test_none() {
        use crate::frontend::None;

        let mut none = None;
        let (tx, rx) = mpsc::channel();
        assert!(none.init(tx, rx).is_ok());
        assert!(none.listen().is_ok());
    }
}
