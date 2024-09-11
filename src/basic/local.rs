use alloc::{collections::vec_deque::VecDeque, rc::Rc};
use core::cell::RefCell;

use crate::*;

#[derive(Debug)]
pub struct MpmcSender<T>(Rc<RefCell<Mpmc<T>>>);

impl<T> Clone for MpmcSender<T> {
    fn clone(&self) -> Self {
        self.0.borrow_mut().send_count += 1;
        Self(self.0.clone())
    }
}

impl<T> Drop for MpmcSender<T> {
    fn drop(&mut self) {
        self.0.borrow_mut().send_count -= 1;
    }
}

impl<T> Connectivity for MpmcSender<T> {
    fn is_connected(&self) -> bool {
        self.0.borrow().recv_count > 0
    }
}

impl<T> Capacity for MpmcSender<T> {
    fn capacity(&self) -> Option<usize> {
        self.0.borrow().capacity
    }
}

impl<T> Sender for MpmcSender<T> {
    type Item = T;

    fn is_full(&self) -> bool {
        let b = self.0.borrow();
        match b.capacity {
            Some(t) => b.queue.len() >= t,
            None => false,
        }
    }

    fn send(&mut self, item: Self::Item) -> Result<(), SendError<Self::Item>> {
        let mut b = self.0.borrow_mut();

        if b.recv_count == 0 {
            return Err(SendError {
                item,
                kind: SendErrorKind::Disconnected,
            });
        }

        if b.capacity.map_or(false, |t| b.queue.len() >= t) {
            return Err(SendError { item, kind: SendErrorKind::Full });
        }

        b.queue.push_back(item);
        Ok(())
    }
}

#[derive(Debug)]
pub struct MpmcReceiver<T>(Rc<RefCell<Mpmc<T>>>);

impl<T> Clone for MpmcReceiver<T> {
    fn clone(&self) -> Self {
        self.0.borrow_mut().recv_count += 1;
        Self(self.0.clone())
    }
}

impl<T> Drop for MpmcReceiver<T> {
    fn drop(&mut self) {
        self.0.borrow_mut().recv_count -= 1;
    }
}

impl<T> Connectivity for MpmcReceiver<T> {
    fn is_connected(&self) -> bool {
        self.0.borrow().send_count > 0
    }
}

impl<T> Capacity for MpmcReceiver<T> {
    fn capacity(&self) -> Option<usize> {
        self.0.borrow().capacity
    }
}

impl<T> Receiver for MpmcReceiver<T> {
    type Item = T;

    fn is_empty(&self) -> bool {
        self.0.borrow().queue.is_empty()
    }

    fn recv(&mut self) -> Result<Self::Item, RecvError> {
        let mut b = self.0.borrow_mut();
        b.queue.pop_front().ok_or_else(|| RecvError {
            kind: if b.send_count == 0 {
                RecvErrorKind::Disconnected
            } else {
                RecvErrorKind::Empty
            },
        })
    }
}

#[derive(Debug)]
pub struct Mpmc<T> {
    queue: VecDeque<T>,
    capacity: Option<usize>,
    recv_count: usize,
    send_count: usize,
}

impl<T> Channel for Mpmc<T> {
    type Item = T;

    type Sender = MpmcSender<T>;
    type Receiver = MpmcReceiver<T>;
}

impl<T> Bounded for Mpmc<T> {
    fn bounded(capacity: usize) -> Pair<Self> {
        let channel = Rc::new(RefCell::new(Mpmc {
            queue: VecDeque::with_capacity(capacity),
            capacity: Some(capacity),
            recv_count: 1,
            send_count: 1,
        }));
        (MpmcSender(channel.clone()), MpmcReceiver(channel))
    }
}

impl<T> Unbounded for Mpmc<T> {
    fn unbounded() -> Pair<Self> {
        let channel = Rc::new(RefCell::new(Mpmc {
            queue: VecDeque::new(),
            capacity: None,
            recv_count: 1,
            send_count: 1,
        }));
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
