use core::marker::PhantomData;

use crate::*;

impl<T> Capacity for crossbeam_channel::Sender<T> {
    fn capacity(&self) -> Option<usize> {
        self.capacity()
    }
}

impl<T> Sender for crossbeam_channel::Sender<T> {
    type Item = T;

    fn is_full(&self) -> bool {
        self.is_full()
    }

    fn send(&mut self, item: Self::Item) -> Result<(), SendError<Self::Item>> {
        self.try_send(item).map_err(|err| match err {
            crossbeam_channel::TrySendError::Full(item) => {
                SendError { kind: SendErrorKind::Full, item }
            }
            crossbeam_channel::TrySendError::Disconnected(item) => SendError {
                kind: SendErrorKind::Disconnected,
                item,
            },
        })
    }
}

impl<T> Capacity for crossbeam_channel::Receiver<T> {
    fn capacity(&self) -> Option<usize> {
        self.capacity()
    }
}

impl<T> Receiver for crossbeam_channel::Receiver<T> {
    type Item = T;

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn recv(&mut self) -> Result<Self::Item, RecvError> {
        self.try_recv().map_err(|err| match err {
            crossbeam_channel::TryRecvError::Empty => RecvError { kind: RecvErrorKind::Empty },
            crossbeam_channel::TryRecvError::Disconnected => RecvError {
                kind: RecvErrorKind::Disconnected,
            },
        })
    }
}

pub struct Crossbeam<T>(PhantomData<(T, fn(T))>);

impl<T> Channel for Crossbeam<T> {
    type Item = T;

    type Sender = crossbeam_channel::Sender<T>;
    type Receiver = crossbeam_channel::Receiver<T>;
}

impl<T> UnboundedChannel for Crossbeam<T> {
    fn unbounded() -> (Self::Sender, Self::Receiver) {
        crossbeam_channel::unbounded()
    }
}

impl<T> BoundedChannel for Crossbeam<T> {
    fn bounded(capacity: usize) -> (Self::Sender, Self::Receiver) {
        crossbeam_channel::bounded(capacity)
    }
}
