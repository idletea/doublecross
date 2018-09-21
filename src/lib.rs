//! Bi-directional channels built on [crossbeam_channel](https://docs.rs/crossbeam-channel/0.2.6/crossbeam_channel/)
//!
//! # Examples
//!
//! ```rust
//! # use doublecross::unbounded;
//! # use std::thread;
//! // doublecross supports different types for each
//! // direction in the channel: the type arguments are
//! // the receive/send types for the left channel, and 
//! // the send/receive types for the right channel.
//! let (left, right) = unbounded::<bool, i32>();
//! thread::spawn(move || {
//!     loop {
//!         let val = right.recv().unwrap();
//!         right.send(val % 2 == 0);
//!     }
//! });
//!
//! for i in 0..10 {
//!     left.send(i);
//!     if left.recv().unwrap() {
//!         println!("{} is even", i);
//!     }
//! }
//! ```
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
/// # Type Arguments
///
/// Unforunately it is often necessary to annotate types to help the compiler;
/// the type `T` refers to the type the left channel receives and the right
/// channel sends, and the type `U` inversely refers to the type the right
/// channel receives and the left channel sends.
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
/// let (left, right) = bounded::<(), ()>(0);
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
pub fn bounded<T, U>(cap: usize) -> (BiChannel<T, U>, BiChannel<U, T>) {
    let (tx1, rx1) = channel::bounded(cap);
    let (tx2, rx2) = channel::bounded(cap);
    (BiChannel::new(tx1, rx2), BiChannel::new(tx2, rx1))
}

/// Creates a bi-directional channel of unbounded capacity.
///
/// This type of channel can hold any number of messages in
/// either direction (ie: it has infinite capacity on both sides)
///
/// # Type Arguments
///
/// Unforunately it is often necessary to annotate types to help the compiler;
/// the type `T` refers to the type the left channel receives and the right
/// channel sends, and the type `U` inversely refers to the type the right
/// channel receives and the left channel sends.
///
/// # Examples
///
/// ```rust
/// # use doublecross::unbounded;
/// let (left, right) = unbounded::<i32, i32>();
/// left.send(10);
/// assert_eq!(right.recv(), Some(10));
/// ```
pub fn unbounded<T, U>() -> (BiChannel<T, U>, BiChannel<U, T>) {
    let (tx1, rx1) = channel::unbounded();
    let (tx2, rx2) = channel::unbounded();
    (BiChannel::new(tx1, rx2), BiChannel::new(tx2, rx1))
}

/// Bi-directional communication build on, and
/// usable with, crossbeam-channel channels.
pub struct BiChannel<T, U> {
    pub rx: Receiver<T>,
    pub tx: Sender<U>,
}

impl<T, U> BiChannel<T, U> {
    pub fn new(tx: Sender<U>, rx: Receiver<T>) -> Self {
        BiChannel { rx, tx }
    }

    pub fn send(&self, msg: U) {
        self.tx.send(msg)
    }

    pub fn recv(&self) -> Option<T> {
        self.rx.recv()
    }
}

impl<'a, T, U> RecvArgument<'a, T> for &'a BiChannel<T, U> {
    type Iter = option::IntoIter<&'a Receiver<T>>;

    fn _as_recv_argument(&'a self) -> Self::Iter {
        Some(&self.rx).into_iter()
    }
}

impl<'a, T, U> SendArgument<'a, T> for &'a BiChannel<U, T> {
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
        let (left, right) = super::bounded::<(), ()>(0);
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
        let (left, right) = super::bounded::<(), ()>(0);
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

    #[test]
    fn asymmetric_message_types() {
        let (left, right) = super::unbounded::<u8, i16>();
        left.send(0i16);
        assert_eq!(right.recv().unwrap(), 0i16);
        right.send(0u8);
        assert_eq!(left.recv().unwrap(), 0u8);
    }
}
