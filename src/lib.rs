#[macro_use]
extern crate crossbeam_channel as channel;
use channel::internal::select::{RecvArgument, SendArgument};
use channel::{Receiver, Sender};
use std::option;

/// Creates a bi-directional channel of bounded capacity.
///
/// This type of channel has an internal buffer of length `cap` in which
/// messages get queued; the buffer for each side of the channel is distinct.
///
/// A rather special case is a zero-capacity channel, also known as a
/// rendezvous channel. Such a channel cannot hold any messages since
/// its buffer is of length zero. Instead, send and receive operations
/// must be executing at the same time in order to pair up and pass
/// the message over.
///
/// # Warning
///
/// No effort is made to prevent a deadlock (ie both sides waiting
/// for a buffer space on the other side) and it is important to
/// use this channel in such a way as to avoid deadlocks, or to
/// recognize their occurence and handle them.
///
/// # Examples
///
/// ```rust
/// # use doublecross::bounded;
/// # use std::thread;
/// let (left, right) = bounded(0);
///
/// thread::spawn(move || {
///     // ...
///     left.send(());
/// });
///
/// println!("waiting for rendezvous");
/// right.recv().unwrap();
/// println!("rendezvous complete");
/// ```
pub fn bounded<T>(cap: usize) -> (BiChannel<T>, BiChannel<T>) {
    let (tx1, rx1) = channel::bounded(cap);
    let (tx2, rx2) = channel::bounded(cap);
    (BiChannel::new(tx1, rx2), BiChannel::new(tx2, rx1))
}

/// Creates a bi-directional channel of unbounded capacity.
///
/// This type of channel can hold any number of messages in
/// either direction (ie: it has infinite capacity on both sides)
///
/// # Examples
///
/// ```rust
/// # use doublecross::unbounded;
/// let (left, right) = unbounded();
/// left.send(10);
/// assert_eq!(right.recv(), Some(10));
/// ```
pub fn unbounded<T>() -> (BiChannel<T>, BiChannel<T>) {
    let (tx1, rx1) = channel::unbounded();
    let (tx2, rx2) = channel::unbounded();
    (BiChannel::new(tx1, rx2), BiChannel::new(tx2, rx1))
}

/// Bi-directional communication build on, and
/// usable with, crossbeam-channel channels.
pub struct BiChannel<T> {
    pub rx: Receiver<T>,
    pub tx: Sender<T>,
}

impl<T> BiChannel<T> {
    pub fn new(tx: Sender<T>, rx: Receiver<T>) -> Self {
        BiChannel { rx, tx }
    }

    pub fn send(&self, msg: T) {
        self.tx.send(msg)
    }

    pub fn recv(&self) -> Option<T> {
        self.rx.recv()
    }
}

impl<'a, T> RecvArgument<'a, T> for &'a BiChannel<T> {
    type Iter = option::IntoIter<&'a Receiver<T>>;

    fn _as_recv_argument(&'a self) -> Self::Iter {
        Some(&self.rx).into_iter()
    }
}

impl<'a, T> SendArgument<'a, T> for &'a BiChannel<T> {
    type Iter = option::IntoIter<&'a Sender<T>>;

    fn _as_send_argument(&'a self) -> Self::Iter {
        Some(&self.tx).into_iter()
    }
}

#[cfg(test)]
mod tests {
    use channel;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn simultaneous_handover() {
        let (left, right) = super::bounded(1);
        left.send(10);
        right.send(20);
        assert_eq!(left.recv(), Some(20));
        assert_eq!(right.recv(), Some(10));
    }

    #[test]
    fn rendezvous_recv_select() {
        let (left, right) = super::bounded(0);
        let timeout = Duration::from_millis(10);

        thread::spawn(move || {
            left.send(());
        });

        select! {
            recv(right, _msg) => {},
            recv(channel::after(timeout)) => {
                panic!("timeout waiting for rendezvous");
            },
        }
    }

    #[test]
    fn rendezvous_send_select() {
        let (left, right) = super::bounded(0);
        let timeout = Duration::from_millis(10);

        thread::spawn(move || {
            left.recv();
        });

        select! {
            send(right, ()) => {},
            recv(channel::after(timeout)) => {
                panic!("timeout waiting for rendezvous");
            },
        }
    }
}
