use super::Result;

pub trait SenderChunk<T> {
    fn recv_all(&self) -> Result<impl Iterator<Item = T> + 'static>;
}

#[cfg(feature = "crossbeam")]
mod internal {
    use super::Result;

    pub use crossbeam::channel::bounded;
    pub use crossbeam::channel::unbounded;

    pub type BlockReceiver<T> = crossbeam::channel::Receiver<T>;
    pub type BlockSender<T> = crossbeam::channel::Sender<T>;
    pub type Sender<T> = crossbeam::channel::Sender<T>;
    pub type Receiver<T> = crossbeam::channel::Receiver<T>;

    impl<T: 'static> super::SenderChunk<T> for BlockReceiver<T> {
        fn recv_all(&self) -> Result<impl Iterator<Item = T> + 'static> {
            let mut result = vec![];
            while let Ok(element) = self.try_recv() {
                result.push(element);
            }
            Ok(result.into_iter())
        }
    }
}

#[cfg(not(feature = "crossbeam"))]
// #[cfg(feature = "crossbeam")]
mod internal {
    use super::{Result, SenderChunk};
    use std::sync::{Arc, Mutex};

    pub type Sender<T> = crossbeam::channel::Sender<T>;
    pub type Receiver<T> = crossbeam::channel::Receiver<T>;

    pub use crossbeam::channel::bounded;

    pub fn unbounded<T>() -> (BlockSender<T>, BlockReceiver<T>) {
        let queue = Arc::new(Mutex::new(Vec::new()));
        let sender = BlockSender::new(queue.clone());
        let receiver = BlockReceiver::new(queue);
        (sender, receiver)
    }

    pub struct BlockSender<T> {
        queue: Arc<Mutex<Vec<T>>>,
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
            }
        }
    }

    impl<T> BlockSender<T> {
        const fn new(queue: Arc<Mutex<Vec<T>>>) -> Self {
            Self { queue }
        }

        pub fn send(&self, message: T) -> Result<()> {
            self.queue.lock().map_err(|_| "cannot lock")?.push(message);

            Ok(())
        }

        pub fn same_channel(&self, _other: &Self) -> bool {
            todo!()
        }
    }

    pub struct BlockReceiver<T> {
        queue: Arc<Mutex<Vec<T>>>,
    }

    impl<T> BlockReceiver<T> {
        const fn new(queue: Arc<Mutex<Vec<T>>>) -> Self {
            Self { queue }
        }
    }
    impl<T: 'static> SenderChunk<T> for BlockReceiver<T> {
        fn recv_all(&self) -> Result<impl Iterator<Item = T> + 'static> {
            let mut queue = self.queue.lock().unwrap();
            let mut result = vec![];
            if !queue.is_empty() {
                std::mem::swap(&mut result, &mut *queue);
                drop(queue);
            }
            Ok(result.into_iter())
        }
    }

    #[cfg(test)]
    mod tests {
        use std::thread;

        use super::*;

        #[test]
        fn test_unbounded_queue() {
            let (sender, receiver) = unbounded::<i32>();

            sender.send(1).unwrap();
            sender.send(2).unwrap();
            sender.send(3).unwrap();

            let messages = receiver.recv_all().unwrap().collect::<Vec<_>>();
            assert_eq!(messages, vec![1, 2, 3]);
        }

        #[test]
        fn test_empty_queue() {
            let (_sender, receiver) = unbounded::<i32>();
            let messages = receiver.recv_all().unwrap().collect::<Vec<_>>();
            assert_eq!(messages, Vec::<i32>::new());
        }

        #[test]
        fn test_multiple_consume() {
            let (sender, receiver) = unbounded::<i32>();

            sender.send(1).unwrap();
            sender.send(2).unwrap();

            let messages1 = receiver.recv_all().unwrap().collect::<Vec<_>>();
            assert_eq!(messages1, vec![1, 2]);

            let messages2 = receiver.recv_all().unwrap().collect::<Vec<_>>();
            assert_eq!(messages2, Vec::<i32>::new());
        }

        #[test]
        fn test_concurrent_push() {
            let (sender, receiver) = unbounded::<i32>();
            let num_threads = 10;
            let messages_per_thread = 100;

            let mut handles = vec![];
            for i in 0..num_threads {
                let sender_clone = BlockSender {
                    queue: sender.queue.clone(),
                };
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
            let (sender, receiver) = unbounded::<i32>();
            let num_threads = 5;
            let messages_per_thread = 20;

            let mut sender_handles = vec![];
            let mut receiver_handles = vec![];

            // Spawn sender threads
            for i in 0..num_threads {
                let sender_clone = BlockSender {
                    queue: sender.queue.clone(),
                };
                sender_handles.push(thread::spawn(move || {
                    for j in 0..messages_per_thread {
                        sender_clone.send(i * messages_per_thread + j).unwrap();
                    }
                }));
            }

            // Spawn receiver threads
            receiver_handles.push(thread::spawn(move || {
                let messages = receiver.recv_all().unwrap().collect::<Vec<_>>();
                assert_eq!(messages.len(), 100);
                let messages = receiver.recv_all().unwrap().collect::<Vec<_>>();
                assert_eq!(messages.len(), 0);
            }));

            // Join all sender threads
            for handle in sender_handles {
                handle.join().unwrap();
            }

            // Join all receiver threads
            for handle in receiver_handles {
                handle.join().unwrap();
            }
        }
    }
}

pub use internal::*;
