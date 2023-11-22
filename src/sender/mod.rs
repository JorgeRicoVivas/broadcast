use core::fmt::{Debug, Formatter};
use core::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::mpsc::{Sender, SendError};

use crate::handler::BroadcastMessage;

pub struct BroadcastSender<T> {
    sender: Sender<BroadcastMessage<T>>,
}

impl<T> BroadcastSender<T> {
    pub(crate) fn new(inner: Sender<BroadcastMessage<T>>) -> BroadcastSender<T> {
        Self { sender: inner }
    }

    pub fn send(&self, value: T) -> Result<(), SendError<BroadcastMessage<T>>> {
        self.sender.send(BroadcastMessage::Value(value))
    }

    pub fn set_send_to_self(&self, send_to_self: bool) -> Result<(), SendError<BroadcastMessage<T>>> {
        self.sender.send(BroadcastMessage::ChangeSendToSelf(send_to_self))
    }
}

impl<T> Clone for BroadcastSender<T> {
    fn clone(&self) -> Self {
        Self { sender: self.sender.clone() }
    }
}

impl<T> Debug for BroadcastSender<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Broadcast sender").finish_non_exhaustive()
    }
}

unsafe impl<T: Send> Send for BroadcastSender<T> {}

unsafe impl<T: Send> Sync for BroadcastSender<T> {}

impl<T> RefUnwindSafe for BroadcastSender<T> {}

impl<T> Unpin for BroadcastSender<T> {}

impl<T> UnwindSafe for BroadcastSender<T> {}