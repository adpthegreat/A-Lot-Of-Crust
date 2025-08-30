use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut inner = self.shared.inner.lock().unwrap(); 
        inner.senders += 1;
        drop(inner); // drop the lock 
        Sender {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut inner = self.shared.inner.lock().unwrap(); 
        inner.senders -= 1;
        let was_last = inner.senders == 0;
        drop(inner); // drop the lock 
        if was_last {
            self.shared.available.notify_one();
        } // if we're the last sender when we drop, then we have to wake up any receivers 
    }
}

impl<T> Sender<T> {
    pub fn send(&mut self, t : T) {
        let mut inner = self.shared.inner.lock().unwrap(); 
        inner.queue.push_back(t);
        drop(inner); // drop the lock 
        self.shared.available.notify_one(); // notify one thread waiting on the Condvar
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
    buffer: VecDeque<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Option<T> {
         if let Some(T) = self.buffer.pop_front() {
            return Some(T)
         }
         let mut inner = self.shared.inner.lock().unwrap(); 
         loop {
            println!("this is no of senders {}",inner.senders);
            match inner.queue.pop_front() {
                Some(t) => {
                    if !inner.queue.is_empty() {
                        std::mem::swap(&mut self.buffer, &mut inner.queue);
                    }
                    return Some(t);
                }
                None if inner.senders == 0 => return None,
                None => {
                    println!("i'm in this block too!");
                    inner = self.shared.available.wait(inner).unwrap();
                } //only if the sender count is > 0 do we want to block
            }
         }
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.recv()
    }
}

struct Inner<T> {
    queue:VecDeque<T>,
    senders: usize,
}
struct Shared<T> {
    inner: Mutex<Inner<T>>,
    available: Condvar,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Inner {
        queue: VecDeque::new(),
        senders: 1,
    };
    let shared = Shared { 
        inner: Mutex::new(inner), 
        available: Condvar::new()
    };
    let shared = Arc::new(shared);
    (
        Sender {
            shared: shared.clone()
        },
        Receiver {
            shared: shared.clone(),
            buffer: VecDeque::new(),
        },
    )
}

#[cfg(test)] 
mod tests {
    use super::*;

    #[test]
    fn ping_pong() {
        let (mut tx, mut rx) = channel();
        tx.send(42);
        assert_eq!(rx.recv(),Some(42))
    }

    #[test]
    fn closed_tx() {
        let (tx, mut rx) = channel::<()>();
        drop(tx);
        assert_eq!(rx.recv(), None);
    }
    #[test]
    fn closed_rx() {
        let (mut tx, mut rx) = channel();
        // drop(tx); // moves it , cuz its dropped obv
        tx.send(42);
        // assert_eq!
    }
    #[test]
    fn test_buffer(){
        let (mut tx, mut rx) = channel();
        let buf = VecDeque::<i32>::new();
        tx.send(42);
        tx.send(69);
        tx.send(81);
        tx.send(007);
        rx.recv();
        assert_eq!(rx.buffer, [69, 81, 007])
    }
}
fn main() {
    println!("Hello, world!");
}


// dbg!() for print debugging 