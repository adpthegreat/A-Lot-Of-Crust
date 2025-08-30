

## Channels!

For commuication obv 

You can clone the Sender but you cannot clone the receiver , hence the name multi producer, single consumer 

Like go channels 

The Sender is `Send` if T is Send , you can construct a channel and send stuff over it that is not `Send` as long as you don't move the thing across the thread boundary.

This is the type signature

```rust 
 impl <T: Send> Send for Sender<T> 
```

- Unless you specify that `T` is `?Sized` then `T` has to be Sized

- the Sender thread is distinguished from the Receiver with types 


The channel Object owns the data in it, when you Send data and you don't receive it, the channel makes sure the data gets dropped

## What will be using for our channel 

- Lock method returns Guard and once you have that guard you are guaranteed to be the only thing that can access the `T` that is protected by the Mutex

- `Arc` Atomically reference counting pointer
We want it to work across thread boundaries, one of the reasons `Mutex` is being used and not `RefCell`

`Condvar` - conditional variable, you use to tell other threads that it has something it cares about that you can access 12.00

Now we can use this to write code in a safe way 

- A common pattern when making data structures in rust , we sometimes use an `Inner<T>` type that holds the data that is shared 

- Both `Mutex` and `RefCell` do runtime borrow checking , the difference is if two threads try to access the same thing at the same time Mutex will block one thread, while `refCell` will not allow you to get it at the momemnt if it is not mutable.

## Why does the receiver type needs an Arc protected by a mutex if the channel might only have one consumer thread

Great question ngl 

- They might have T at the same time (how?)

- Send and Receive need to be mutually exclusive to each other so they all need to be synchronized with the mutex

- A mutex is a basically bool semaphore, 

The low level differences between the mutex and the semaphore being 

- Mutex - parking mode mechanisms and user mode futexes that are impl by the OS.

- Boolean semaphore, boolean flag that can be atomically updated - if someone else is in the critical section that is if has the lock and is setting the flag, then you have to spin and comtinuously check if you can access it .

- Also for the mutex the OS can make the thread sleep and wake it up when the mutex is available, which is more efficent but has more latency 

## Why Arc ?

if there was no Arc then the `Sender` and `receiver` would have difference instances of `Inner` and if so, how can they communicate? 

```rust 
use std::sync::{Arc, Mutex};

pub struct Sender<T> {
    inner: Inner<T>,
}

pub struct Receiver<T> {
    inner: Inner<T>,
}

```

they made `LockResult` generic so its not a guard type in the result anymore, it has changed since the video;
old

```rust
pub type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;
```

```rust
pub type LockResult<T> = Result<T, PoisonError<T>>;
```

## Using Vecs for channels

- We're using `Vec` like a stack, if the sender pushes twice and the receiver pops then it will get the last element and not the first element - which we obviously don't want , we want to get the data that the channels sends first.

- We can solve this by removing the first element in the Vec, but the issue is when remove an element from the Vec then the other elements have to move their position to fill in the gap , that is it has to resize, so we are not going to use Vec for this, we would use `VecDeque` - the closest thing to ringBuffers in the rust `std::collections` 

## Using VecDeque

- like a vector - both it keeps track of the start and end points 

- You don't want to use swap_remove instead of pop_front in the receiver because that makes the last things sent become the first thing to be received, it changes the order of the elements - order matters

- pop_front returns an `Option<T>` because its possible for there not to be anything in the VecDeque, we can implement a `try_recv` method that uses this but we want a blocking version of `recv`, one that waits until there is something in the channel to implement the blocking recv we'll use a `CondVar`.

## No Condvars inside the Mutex

- The Condvar has to be outside the Mutex , the reason being to prevent scenarios where a thread is holding the lock for the Mutex with the `Condvar` inside and you have to wake other threads up, when you wake them up and they try to get the lock, they go to sleep while you keep on running, when you eventually release the lock, no thread is awake, you end up with a deadlock.

- So what you do is to release the lock on the Mutex, the same time you notify the other threads

- Looking at the `wait` method on Condvar, it returns a `MutexGuard` and this makes sense because when a thread is notified or woken up from sleep it has to acquire the lock from the `Mutex`

- The Sender has to notify the receiver

## Implementation Problem 

- We need a way to indicate to the receiver that there are no more Senders left, with this current impl of the receiver there is a problem 

```rust 
impl<T> Receiver<T> {
    pub fn recv(&mut self) -> T {
         let mut queue = self.inner.queue.lock().unwrap(); 
         loop {
            match queue.pop_front() {
                Some(t) => return t,
                None => {
                    queue = self.inner.available.wait(queue).unwrap();
                }
            }
         }
    }
}
```

Lets write a test to show the problem as we can see, it just waits forever (well the test closes after 60 seconds by the testing suire after no activity)

```rust 
   Compiling channels v0.1.0 (/Users/me/Desktop/A-Lot-Of-Crust/channels)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.05s
     Running unittests src/main.rs (target/debug/deps/channels-26b59efaa373179f)

running 2 tests
test tests::ping_pong ... ok
test tests::closed has been running for over 60 seconds
```

## Solution that still had a problem 

- We solve this by refctoring out code and creating two structs, one named `Shared` and the other named `Inner` , the shared struct holds our Mutex of the inner struct and our Condvar while the inner struct now holds the number of `senders` as a `usize` and the `VecDeque` object so we've effectively separated the shared resource we want to access from the from the synchronization primitives to control access to them.

- We also write impls for `Clone` and `Drop` traits, in the logic we decrement or increment the number of senders, if the `Sender` is cloned or dropped respectively 

```rust 


```

- In the recv method of the receiver , we first checked if the number of senders was 0 and then we return None , but this didn't solve the waiting issue where our test hung, its not a good way to check fi there are no senders.

```rust 
 match inner.queue.pop_front() {
                Some(t) => return Some(t),
                None if inner.senders == 0 => { //here
                    println!("this is no of senders {}",inner.senders);
                    return None
                },
                None => {
                    println!("i'm in this block too!");
                    inner = self.shared.available.wait(inner).unwrap();
                } 
            }
```

Instead lets use a strong_count fo the Arc shared reference, we would check if there is only 1 reference, that is the one made by the receiver then there are no senders and we can return None and close the channels.

So we don't need the senders field anymore and it can be removed from the `Inner` struct 

```rust 

struct Inner<T> {
    queue:VecDeque<T>,
    senders: usize, // we can remove this now
}
```

we can now remove the logic for incrementing and decrementing the sender count in bot h the Clone trait impls

```rust 
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
```

```rust
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
```

but this causes another issue in the drop trait impl

if you drop a sender then you dont know whether to notify 

48.10 complete it


When theres a last sender, then theres at most one receiver left so we don't need to notify all 

```rust
 if was_last {
    self.shared.available.notify_one();
} 
```

## Drop test

Turns out the drop method was not invoked, assigiing an underscore to the tx didn't invoke it, if we dropped it manually the test passed and the receiever did not wait 

```rust 
 #[test]
    fn closed() {
        let (tx, mut rx) = channel::<()>();
        drop(tx);
        // let _= tx; // does not drop invoke
        assert_eq!(rx.recv(), None);
    }
```

its similar to the std lib impl too

```rust 
use std::sync::mpsc::sync_channel;
use std::thread;

let (tx, rx) = sync_channel(3);

for _ in 0..3 {
    // It would be the same without thread and clone here
    // since there will still be one `tx` left.
    let tx = tx.clone();
    // cloned tx dropped within thread
    thread::spawn(move || tx.send("ok").unwrap());
}

// Drop the last sender to stop `rx` waiting for message.
// The program will not complete if we comment this out.
// **All** `tx` needs to be dropped for `rx` to have `Err`.
drop(tx);

```

```rust 
//add flag to the Inner T, when the receiver drops the flag is set and notify_all is called

// if sender tries to send and error is thrown rather than pushing to the queue
```

- If i try to send something and the receiver has been closed i should be told that, whether or not you want this is a design decision, here we are not going to do that.

## Critiquing design decisions and potential improvements

- `All operations take the lock` - in a high performance scenario,
we would not want the Senders to content with one another, its only the sender and the receiver that the synchronization should occur between 

- `Synchronized or unsynchronized Senders`  - Looking at the std lib channel impl, we can see that we have to sender types `Sender` and `SyncSender` the difference between that is that `Sender` is async and `SyncSender` is sync , but not in the conventional async await way , what we mean is they are forced to synchronize, that is when one is faster than the other , in our current design the sender can send as many as they want to the receiver and the queues capacity increases, if its a sync sender way, thre is a limited capacity it can send at first, if it exceeds that then the Sender blocks

- The primary difference is whether the `Sender` can block or not 

- In the std lib `sync_channel` takes a bound of type `usize`

```rust 
pub fn sync_channel(bound:usize) -> (SyncSender<T>, Receiver<T>)
```

- How we would implement it in our context? We would use Two Condvars - one for notifying the senders and one for notifying the receivers 

## Question 
Q - How would you do an Iterator impl that consumes values from channel until all senders are gone and it ends with None

```rust
impl<T> Iterator for Receiver {
    type Item = T;
    fn next(&mut self) -> Option<Self::item> {
        self.recv;
    }
} 
```

## Optimizing our channel by using a buffer

- Since theres only one receiver, we don't need to take the lock for every receive so lets add a likkle optimization (in scarface's voice)

- What we do is that we can steal all the current processes / items in the channel and put in a local buffer instead of stealing it one by one, so that we wouldn't need to call the lock everytiime and just access the buffer everythime we call recv

Heres the new recv implementation with the buffer included

```rust 
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
                } 
            }
         }
    }
}

```
So the process would be like 
```md
invoke recv 
-> check if theres items in the buffer last time we took the lock 
    -> if yes then take the first item (pop_front) 
    -> if no then take the lock and enter the loop
        -> try to access the first item in the queue
            -> if the queue is not empty swap the elements of the queue and the buffer (so in subsequent calls we dont need to acquire the lock and access the queue....)
            -> if the queue is empty return the element
            // then the rest of the logic 
```

- Draw back is that it uses more memory and more resizes

- But it also reduces the amount of [lock contention](https://en.wikipedia.org/wiki/Lock_(computer_science)#:~:text=lock%20contention%3A%20this%20occurs%20whenever,lock%20held%20by%20the%20other.) - the lock is not taken as many times and the lock is faster to acquire 

- Removing the if condition for the swap does not necessarily make it faster, CPU already has branch Prediction so it can speculatively execute

- No need to return a list - if we return a list we would have to allocated the list everytime 


## Flavors 

Different impls

```md
Synchronous - Channel where send() can block limited capacity 
    -> Mutex + Condvar: VecDeque
    -> Atomic VecDeque (updates the head and tail atomically, so no need for Mutex) + thread::park + thread::Thread::notify 

Asynchronous - Channel where send() cannot block. Unbounded
    -> Mutex + Condvar + VecDeque
    -> Mutex + Condvar + DoublyLinkedList (no resizing and memory problems)
    -> Atomic Linked List, linked list of T
    -> Atomic block linked list, linked list of atomic VecDeque<T>
Rendezvous - Synchronous with capacity = 0 Used for thread synchronization, more like a Condvar and not a mutex 
Oneshot channels - Any capacity, In practice, only one call to send()
 - eg channel to tell all teh threads to exit early
```

- Flume is better where contention is lower , crossbeam is better for high contention cases 
https://docs.rs/flume/latest/flume/

## Further Studying 

### Study std lib mpsc and mpmc impls 

https://doc.rust-lang.org/std/sync/mpmc/index.html
https://doc.rust-lang.org/std/sync/mpsc/index.html

#### Study crossbeam and tokio mpsc impls 

https://docs.rs/crossbeam/0.8.4/crossbeam/channel/index.html
https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html

### Study VecDeque

https://doc.rust-lang.org/src/alloc/collections/vec_deque/drain.rs.html 

### Study Condvar

https://doc.rust-lang.org/src/std/sync/poison/condvar.rs.html

### Study the stuff in this 

https://www.reddit.com/r/rust/comments/12cpdgr/why_does_synccondvar_require_a_mutex/

Chapter 5 or mara bos' Rust atomics and locks 
https://marabos.nl/atomics/ 

https://en.wikipedia.org/wiki/Circular_buffer

### Study lock free algos 
https://en.wikipedia.org/wiki/Non-blocking_algorithm