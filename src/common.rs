use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SendErrorKind {
    Full,
    Disconnected,
}

impl fmt::Display for SendErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendErrorKind::Full => write!(f, "channel is full"),
            SendErrorKind::Disconnected => write!(f, "channel is disconnected"),
        }
    }
}

#[derive(Debug)]
pub struct SendError<T> {
    pub kind: SendErrorKind,
    pub item: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecvErrorKind {
    Empty,
    Disconnected,
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            RecvErrorKind::Empty => write!(f, "channel is empty"),
            RecvErrorKind::Disconnected => write!(f, "channel is disconnected"),
        }
    }
}

#[derive(Debug)]
pub struct RecvError {
    pub kind: RecvErrorKind,
}

pub trait Connectivity {
    fn is_connected(&self) -> bool;
}

pub trait Capacity {
    fn capacity(&self) -> Option<usize>;
}

pub trait Sender {
    type Item;

    fn is_full(&self) -> bool;

    fn send(&mut self, item: Self::Item) -> Result<(), SendError<Self::Item>>;
}

pub trait Receiver {
    type Item;

    fn is_empty(&self) -> bool;

    fn recv(&mut self) -> Result<Self::Item, RecvError>;
}

pub trait Channel {
    type Item;

    type Sender: Sender<Item = Self::Item>;
    type Receiver: Receiver<Item = Self::Item>;
}

pub type Pair<C> = (<C as Channel>::Sender, <C as Channel>::Receiver);

pub trait Bounded: Channel {
    fn bounded(capacity: usize) -> Pair<Self>;
}

pub trait Unbounded: Channel {
    fn unbounded() -> Pair<Self>;
}
