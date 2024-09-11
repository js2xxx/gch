use core::marker::PhantomData;

use crate::*;

impl<T> Connectivity for ::flume::Sender<T> {
    fn is_connected(&self) -> bool {
        !self.is_disconnected()
    }
}

impl<T> Capacity for ::flume::Sender<T> {
    fn capacity(&self) -> Option<usize> {
        self.capacity()
    }
}

impl<T> Sender for ::flume::Sender<T> {
    type Item = T;

    fn is_full(&self) -> bool {
        self.is_full()
    }

    fn send(&mut self, item: Self::Item) -> Result<(), SendError<Self::Item>> {
        self.try_send(item).map_err(|err| match err {
            ::flume::TrySendError::Full(item) => SendError { kind: SendErrorKind::Full, item },
            ::flume::TrySendError::Disconnected(item) => SendError {
                kind: SendErrorKind::Disconnected,
                item,
            },
        })
    }
}

impl<T> Connectivity for ::flume::Receiver<T> {
    fn is_connected(&self) -> bool {
        !self.is_disconnected()
    }
}

impl<T> Capacity for ::flume::Receiver<T> {
    fn capacity(&self) -> Option<usize> {
        self.capacity()
    }
}

impl<T> Receiver for ::flume::Receiver<T> {
    type Item = T;

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn recv(&mut self) -> Result<Self::Item, RecvError> {
        self.try_recv().map_err(|err| match err {
            ::flume::TryRecvError::Empty => RecvError { kind: RecvErrorKind::Empty },
            ::flume::TryRecvError::Disconnected => RecvError {
                kind: RecvErrorKind::Disconnected,
            },
        })
    }
}

pub struct Flume<T>(PhantomData<(T, fn(T))>);

impl<T> Channel for Flume<T> {
    type Item = T;
    type Sender = ::flume::Sender<T>;
    type Receiver = ::flume::Receiver<T>;
}

impl<T> Bounded for Flume<T> {
    fn bounded(capacity: usize) -> Pair<Self> {
        ::flume::bounded(capacity)
    }
}

impl<T> Unbounded for Flume<T> {
    fn unbounded() -> Pair<Self> {
        ::flume::unbounded()
    }
}
