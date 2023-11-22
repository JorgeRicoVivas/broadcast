use core::fmt;
use core::fmt::{Debug, Formatter};
use core::iter::Map;
use core::panic::{RefUnwindSafe, UnwindSafe};
use core::time::Duration;
use std::sync::mpsc::{IntoIter, Iter, Receiver, RecvError, RecvTimeoutError, TryIter, TryRecvError};

use crate::handler::BroadcastMessage;

pub struct BroadcastReceiver<T> {
    receiver: Receiver<BroadcastMessage<T>>,
}

impl<T> BroadcastReceiver<T> {
    pub(crate) fn new(inner: Receiver<BroadcastMessage<T>>) -> BroadcastReceiver<T> {
        Self { receiver: inner }
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.receiver.try_recv().map(BroadcastMessage::<T>::force_value)
    }

    pub fn recv(&self) -> Result<T, RecvError> {
        self.receiver.recv().map(BroadcastMessage::<T>::force_value)
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout).map(BroadcastMessage::<T>::force_value)
    }

    pub fn iter(&self) -> Map<Iter<'_, BroadcastMessage<T>>, fn(BroadcastMessage<T>) -> T> {
        self.receiver.iter().map(BroadcastMessage::<T>::force_value)
    }

    pub fn try_iter(&self) -> Map<TryIter<'_, BroadcastMessage<T>>, fn(BroadcastMessage<T>) -> T> {
        self.receiver.try_iter().map(BroadcastMessage::<T>::force_value)
    }
}

impl<T> Debug for BroadcastReceiver<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Broadcast receiver").finish_non_exhaustive()
    }
}

impl<'a, T> IntoIterator for &'a BroadcastReceiver<T> {
    type Item = T;
    type IntoIter = Map<Iter<'a, BroadcastMessage<T>>, fn(BroadcastMessage<T>) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for BroadcastReceiver<T> {
    type Item = T;
    type IntoIter = Map<IntoIter<BroadcastMessage<T>>, fn(BroadcastMessage<T>) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        self.receiver.into_iter().map(BroadcastMessage::<T>::force_value)
    }
}

unsafe impl<T: Send> Send for BroadcastReceiver<T> {}

impl<T> RefUnwindSafe for BroadcastReceiver<T> {}

impl<T> Unpin for BroadcastReceiver<T> {}

impl<T> UnwindSafe for BroadcastReceiver<T> {}
