//! Queue module providing bounded and unbounded communication channels.
//!
//! This module implements two types of queues for inter-thread communication:
//! - Bounded queues: Single-item blocking channels
//! - Unbounded queues: Multi-item non-blocking channels

use super::Result;

use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};

/// Creates a bounded queue channel where the receiver blocks until a value is sent.
///
/// This is a single-slot channel: sending will overwrite any previous unsent value,
/// and receiving blocks until a value is available.
///
/// # Type Parameters
///
/// * `T` - The type of values to send through the channel.
///
/// # Returns
///
/// A tuple of (Sender<T>, Receiver<T>) for communicating between threads.
pub fn bounded<T>() -> (Sender<T>, Receiver<T>) {
    let value = Arc::new((Mutex::new(None), Condvar::new()));

    (Sender(value.clone()), Receiver(value))
}

/// Sender for a bounded queue channel.
///
/// Allows sending values to the corresponding receiver. Sending is non-blocking
/// and will overwrite any previous value that hasn't been received yet.
pub struct Sender<T>(Arc<(Mutex<Option<T>>, Condvar)>);

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Sender<T> {
    /// Sends a value through the channel.
    ///
    /// This operation is non-blocking. If a previous value hasn't been received yet,
    /// it will be overwritten.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to send.
    ///
    /// # Errors
    ///
    /// Returns an error if the internal mutex cannot be locked.
    pub fn send(&self, value: T) -> Result<()> {
        let (lock, cvar) = &*self.0;
        let _ = lock.lock().map_err(|_| "cannot lock")?.insert(value);
        cvar.notify_one();
        Ok(())
    }
}

/// Receiver for a bounded queue channel.
///
/// Provides blocking and non-blocking receive operations for values sent
/// through the corresponding sender.
pub struct Receiver<T>(Arc<(Mutex<Option<T>>, Condvar)>);

impl<T> Receiver<T> {
    /// Receives a value from the channel, blocking until one is available.
    ///
    /// # Errors
    ///
    /// Returns an error if the internal mutex cannot be locked.
    pub fn recv(&self) -> Result<T> {
        let (lock, cvar) = &*self.0;
        let mut value = lock.lock().map_err(|_| "cannot lock")?;
        loop {
            match value.take() {
                None => value = cvar.wait(value).map_err(|_| "cannot lock cvar")?,
                Some(v) => return Ok(v),
            }
        }
    }

    /// Attempts to receive a value from the channel without blocking.
    ///
    /// # Returns
    ///
    /// * `Some(value)` if a value was available.
    /// * `None` if no value is currently available.
    ///
    /// # Errors
    ///
    /// Returns an error if the internal mutex cannot be locked.
    pub fn try_recv(&self) -> Result<Option<T>> {
        let (lock, _cvar) = &*self.0;
        let mut value = lock.lock().map_err(|_| "cannot lock")?;

        Ok(value.take())
    }
}

#[cfg(test)]
mod bounded_tests {
    use super::*;
    use std::thread;

    #[test]
    fn it_works() -> Result<()> {
        let (tx, rx) = bounded();

        tx.send(10)?;
        let value = rx.recv()?;

        assert_eq!(value, 10);

        Ok(())
    }

    #[test]
    fn it_try_works() -> Result<()> {
        let (tx, rx) = bounded();

        let value = rx.try_recv()?;
        assert_eq!(value, None);
        tx.send(10)?;

        let value = rx.try_recv()?;
        assert_eq!(value, Some(10));

        let value = rx.try_recv()?;
        assert_eq!(value, None);

        Ok(())
    }

    #[test]
    fn it_works_with_threads() -> Result<()> {
        let (tx, rx) = bounded();

        let tx_thread = tx.clone();
        thread::spawn(move || {
            tx_thread.send(10).unwrap();
        });

        let value = rx.recv()?;

        assert_eq!(value, 10);

        Ok(())
    }
}

/// Creates an unbounded queue channel for multi-item communication.
///
/// This channel allows sending multiple values that accumulate until received.
/// The receiver can retrieve all available values at once.
///
/// # Type Parameters
///
/// * `T` - The type of values to send through the channel.
///
/// # Returns
///
/// A tuple of (`BlockSender`<T>, `BlockReceiver`<T>) for communicating between threads.
pub fn unbounded<T>() -> (BlockSender<T>, BlockReceiver<T>) {
    let queue = Arc::new(Mutex::new(Vec::new()));
    let empty = Arc::new(AtomicBool::new(true));
    let sender = BlockSender::new(queue.clone(), empty.clone());
    let receiver = BlockReceiver::new(queue, empty);
    (sender, receiver)
}

/// Sender for an unbounded queue channel.
///
/// Allows sending multiple values that accumulate in the channel.
/// Sending is non-blocking and thread-safe.
pub struct BlockSender<T> {
    queue: Arc<Mutex<Vec<T>>>,
    empty: Arc<AtomicBool>,
}

impl<T> ::std::fmt::Debug for BlockSender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockSender").finish()
    }
}

impl<T> Clone for BlockSender<T> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            empty: self.empty.clone(),
        }
    }
}

impl<T> BlockSender<T> {
    const fn new(queue: Arc<Mutex<Vec<T>>>, empty: Arc<AtomicBool>) -> Self {
        Self { queue, empty }
    }

    /// Sends a value through the unbounded channel.
    ///
    /// Values accumulate until received by the receiver.
    ///
    /// # Arguments
    ///
    /// * `message` - The value to send.
    ///
    /// # Errors
    ///
    /// Returns an error if the internal mutex cannot be locked.
    pub fn send(&self, message: T) -> Result<()> {
        self.queue.lock().map_err(|_| "cannot lock")?.push(message);
        self.empty.store(false, Ordering::Release);

        Ok(())
    }

    /// Checks if two senders belong to the same channel.
    ///
    /// # Arguments
    ///
    /// * `other` - Another sender to compare with.
    ///
    /// # Returns
    ///
    /// `true` if both senders operate on the same channel, `false` otherwise.
    pub fn same_channel(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.queue, &other.queue)
    }
}

/// Receiver for an unbounded queue channel.
///
/// Provides operations to receive all accumulated values at once.
/// Receiving is non-blocking if no values are available.
pub struct BlockReceiver<T> {
    queue: Arc<Mutex<Vec<T>>>,
    empty: Arc<AtomicBool>,
    buf: Vec<T>,
}

impl<T> BlockReceiver<T> {
    const fn new(queue: Arc<Mutex<Vec<T>>>, empty: Arc<AtomicBool>) -> Self {
        Self {
            queue,
            empty,
            buf: vec![],
        }
    }
}
impl<T: 'static> BlockReceiver<T> {
    /// Receives all currently available values from the channel.
    ///
    /// This operation is non-blocking. If no values are available, returns `None`.
    /// Otherwise, returns an iterator over all accumulated values.
    ///
    /// # Returns
    ///
    /// * `Some(iterator)` containing all available values.
    /// * `None` if no values are currently available.
    pub fn recv_all(&mut self) -> Option<impl Iterator<Item = T> + use<'_, T>> {
        if self.empty.load(Ordering::Acquire) {
            return None;
        }
        let mut queue = self.queue.lock().unwrap();

        std::mem::swap(&mut self.buf, &mut *queue);
        self.empty.store(true, Ordering::Relaxed);
        drop(queue);

        Some(self.buf.drain(..))
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[test]
    fn test_unbounded_queue() {
        let (sender, mut receiver) = unbounded::<i32>();

        sender.send(1).unwrap();
        sender.send(2).unwrap();
        sender.send(3).unwrap();

        let messages = receiver.recv_all().unwrap().collect::<Vec<_>>();
        assert_eq!(messages, vec![1, 2, 3]);
    }

    #[test]
    fn test_empty_queue() {
        let (_sender, mut receiver) = unbounded::<i32>();
        let messages = receiver.recv_all();
        assert!(messages.is_none());
    }

    #[test]
    fn test_multiple_consume() {
        let (sender, mut receiver) = unbounded::<i32>();

        sender.send(1).unwrap();
        sender.send(2).unwrap();

        let messages = receiver.recv_all().unwrap().collect::<Vec<_>>();
        assert_eq!(messages, vec![1, 2]);

        let messages = receiver.recv_all();
        assert!(messages.is_none());
    }

    #[test]
    fn test_concurrent_push() {
        let (sender, mut receiver) = unbounded::<i32>();
        let num_threads = 10;
        let messages_per_thread = 100;

        let mut handles = vec![];
        for i in 0..num_threads {
            let sender_clone = sender.clone();

            handles.push(thread::spawn(move || {
                for j in 0..messages_per_thread {
                    sender_clone.send(i * messages_per_thread + j).unwrap();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let messages = receiver.recv_all().unwrap().collect::<Vec<_>>();
        assert_eq!(messages.len() as i32, num_threads * messages_per_thread);

        let mut sorted_messages = messages.clone();
        sorted_messages.sort();
        for i in 0..(num_threads * messages_per_thread) as i32 {
            assert_eq!(sorted_messages[i as usize], i as i32);
        }
    }

    #[test]
    fn test_concurrent_push_consume() {
        let (sender, mut receiver) = unbounded::<i32>();
        let num_threads = 5;
        let messages_per_thread = 20;

        let mut sender_handles = vec![];
        let mut receiver_handles = vec![];

        // Spawn sender threads
        for i in 0..num_threads {
            let sender_clone = sender.clone();

            sender_handles.push(thread::spawn(move || {
                for j in 0..messages_per_thread {
                    sender_clone.send(i * messages_per_thread + j).unwrap();
                }
            }));
        }

        // Join all sender threads
        for handle in sender_handles {
            handle.join().unwrap();
        }

        // Spawn receiver threads
        receiver_handles.push(thread::spawn(move || {
            let messages = receiver.recv_all().unwrap().collect::<Vec<_>>();
            assert_eq!(messages.len(), 100);
            let messages = receiver.recv_all();
            assert!(messages.is_none());
        }));

        // Join all receiver threads
        for handle in receiver_handles {
            handle.join().unwrap();
        }
    }
}
