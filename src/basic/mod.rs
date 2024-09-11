use alloc::sync::Arc;
use core::sync::atomic::{AtomicUsize, Ordering::SeqCst};

use crossbeam_queue::{ArrayQueue, SegQueue};

use crate::*;

mod local;
pub use self::local::{
    Mpmc as LocalMpmc, MpmcReceiver as LocalMpmcReceiver, MpmcSender as LocalMpmcSender,
};

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum Queue<T> {
    Bounded(ArrayQueue<T>),
    Unbounded(SegQueue<T>),
}

#[derive(Debug)]
pub struct MpmcSender<T>(Arc<Mpmc<T>>);

impl<T> Clone for MpmcSender<T> {
    fn clone(&self) -> Self {
        let arc = self.0.clone();
        arc.send_count.fetch_add(1, SeqCst);
        Self(arc)
    }
}

impl<T> Drop for MpmcSender<T> {
    fn drop(&mut self) {
        self.0.send_count.fetch_sub(1, SeqCst);
    }
}

impl<T> Connectivity for MpmcSender<T> {
    fn is_connected(&self) -> bool {
        self.0.recv_count.load(SeqCst) > 0
    }
}

impl<T> Capacity for MpmcSender<T> {
    fn capacity(&self) -> Option<usize> {
        match self.0.queue {
            Queue::Bounded(ref queue) => Some(queue.capacity()),
            Queue::Unbounded(_) => None,
        }
    }
}

impl<T> Sender for MpmcSender<T> {
    type Item = T;

    fn is_full(&self) -> bool {
        match self.0.queue {
            Queue::Bounded(ref queue) => queue.is_full(),
            Queue::Unbounded(_) => false,
        }
    }

    fn send(&mut self, item: Self::Item) -> Result<(), SendError<Self::Item>> {
        if !self.is_connected() {
            return Err(SendError {
                kind: SendErrorKind::Disconnected,
                item,
            });
        }

        match self.0.queue {
            Queue::Bounded(ref queue) => queue
                .push(item)
                .map_err(|item| SendError { kind: SendErrorKind::Full, item })?,
            Queue::Unbounded(ref queue) => queue.push(item),
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct MpmcReceiver<T>(Arc<Mpmc<T>>);

impl<T> Clone for MpmcReceiver<T> {
    fn clone(&self) -> Self {
        let arc = self.0.clone();
        arc.recv_count.fetch_add(1, SeqCst);
        Self(arc)
    }
}

impl<T> Drop for MpmcReceiver<T> {
    fn drop(&mut self) {
        self.0.recv_count.fetch_sub(1, SeqCst);
    }
}

impl<T> Connectivity for MpmcReceiver<T> {
    fn is_connected(&self) -> bool {
        self.0.send_count.load(SeqCst) > 0
    }
}

impl<T> Capacity for MpmcReceiver<T> {
    fn capacity(&self) -> Option<usize> {
        match self.0.queue {
            Queue::Bounded(ref queue) => Some(queue.capacity()),
            Queue::Unbounded(_) => None,
        }
    }
}

impl<T> Receiver for MpmcReceiver<T> {
    type Item = T;

    fn is_empty(&self) -> bool {
        match self.0.queue {
            Queue::Bounded(ref queue) => queue.is_empty(),
            Queue::Unbounded(ref queue) => queue.is_empty(),
        }
    }

    fn recv(&mut self) -> Result<Self::Item, RecvError> {
        match self.0.queue {
            Queue::Bounded(ref queue) => queue.pop(),
            Queue::Unbounded(ref queue) => queue.pop(),
        }
        .ok_or_else(|| RecvError {
            kind: if self.is_connected() {
                RecvErrorKind::Empty
            } else {
                RecvErrorKind::Disconnected
            },
        })
    }
}

#[derive(Debug)]
pub struct Mpmc<T> {
    queue: Queue<T>,
    recv_count: AtomicUsize,
    send_count: AtomicUsize,
}

impl<T> Channel for Mpmc<T> {
    type Item = T;

    type Sender = MpmcSender<T>;
    type Receiver = MpmcReceiver<T>;
}

impl<T> Unbounded for Mpmc<T> {
    fn unbounded() -> Pair<Self> {
        let channel = Arc::new(Mpmc {
            queue: Queue::Unbounded(SegQueue::new()),
            recv_count: AtomicUsize::new(1),
            send_count: AtomicUsize::new(1),
        });
        (MpmcSender(channel.clone()), MpmcReceiver(channel))
    }
}

impl<T> Bounded for Mpmc<T> {
    fn bounded(capacity: usize) -> Pair<Self> {
        let channel = Arc::new(Mpmc {
            queue: Queue::Bounded(ArrayQueue::new(capacity)),
            recv_count: AtomicUsize::new(1),
            send_count: AtomicUsize::new(1),
        });
        (MpmcSender(channel.clone()), MpmcReceiver(channel))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Receiver, Sender, Unbounded};

    #[test]
    fn test_channel() {
        let (mut tx, mut rx) = Mpmc::unbounded();
        tx.send(1).unwrap();
        assert_eq!(rx.recv().unwrap(), 1);
    }

    #[test]
    fn test_channel_disconnect() {
        let (mut tx, rx) = Mpmc::unbounded();
        drop(rx);
        assert_eq!(
            tx.send(1).unwrap_err().kind,
            crate::SendErrorKind::Disconnected
        );

        let (tx, mut rx) = Mpmc::<()>::unbounded();
        drop(tx);
        assert_eq!(
            rx.recv().unwrap_err().kind,
            crate::RecvErrorKind::Disconnected
        );
    }
}
