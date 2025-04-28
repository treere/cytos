#[cfg(not(feature = "own_queue"))]
mod cross {
    pub use crossbeam::channel::bounded;
    pub use crossbeam::channel::unbounded;
    pub use crossbeam::channel::Receiver;
    pub use crossbeam::channel::Sender;
}

#[cfg(feature = "own_queue")]
mod own {
    use crate::Result;
    use std::sync::{Arc, Mutex};

    pub struct Sender<T> {
        data: Arc<Mutex<Vec<T>>>,
    }

    impl<T> std::fmt::Debug for Sender<T> {
        fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
            write!(w, "Sender")
        }
    }

    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            Self {
                data: self.data.clone(),
            }
        }
    }

    impl<T> Sender<T> {
        pub fn send(&self, value: T) -> Result<()> {
            self.data.lock().map_err(|_| "cannot send")?.push(value);
            Ok(())
        }

        pub fn same_channel(&self, other: &Self) -> bool {
            std::ptr::eq(Arc::as_ptr(&self.data), Arc::as_ptr(&other.data))
        }
    }

    pub struct Receiver<T> {
        data: Arc<Mutex<Vec<T>>>,
    }

    impl<T> Clone for Receiver<T> {
        fn clone(&self) -> Self {
            Self {
                data: self.data.clone(),
            }
        }
    }

    impl<T> Receiver<T> {
        pub fn recv(&self) -> Result<T> {
            loop {
                let mut data = self.data.lock().map_err(|_| "cannot send")?;
                if let Some(v) = data.pop() {
                    return Ok(v);
                }
            }
        }

        pub fn try_recv(&self) -> Result<T> {
            let mut data = self.data.lock().map_err(|_| "cannot send")?;
            data.pop()
                .map_or_else(|| Err("No elements".into()), |v| Ok(v))
        }
    }

    pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
        let data = Arc::new(Mutex::new(Vec::new()));
        let sender = data.clone();

        let receiver = data;

        (Sender { data: sender }, Receiver { data: receiver })
    }

    pub fn bounded<T>(_: usize) -> (Sender<T>, Receiver<T>) {
        unbounded()
    }
}

#[cfg(not(feature = "own_queue"))]
pub use cross::*;

#[cfg(feature = "own_queue")]
pub use own::*;
