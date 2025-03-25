
# What is Send + Sync ?

- used to represent thread safety at the type level

- They are both marker traits <https://doc.rust-lang.org/std/marker/index.html> marker traits have no methods they are used to signify if a type meets or has a given property

- auto traits - the compiler automatically impls the traits for you

- all auto traits are marker traits but not all marker traits are auto traits

- primarily used in trait bounds

## Send

its ok to pass this value to another thread , give up ownership to another thread - it can do whatever it wants , most types in general are Send eg a primitive like a number , boolean or string

### Types that are not Send

- if it might violate some implicit assumptions or invariants that the type has eg `Rc` the atomically reference counted pointer,  
- `MutexGuard` theres a requirement on OSs that the thread that gets the lock is the same thread that releases the lock (so it can't be `Send` obviously ) so its the guard not the mutex itself thats not `Send`, same thread that acquires the lock is the same thread that releases the lock

- Check out the impl of `Mutex<T>` in tokio and the constraints on when its Send or not https://github.com/tokio-rs/tokio/blob/master/tokio/src/sync/mutex.rs#L255 

### Understanding Rc and why it is not Send

In our `main.rs` we wrote a simple implementation of Rc<T> implementing some basic traits like `Clone`, `Send`, and `Deref` traits, we have our Inner Struct which holds a value and a count that increments everytime Rc is referenced, below is a code snippet for the the impl of the `Clone` trait

```rust
impl<T> Clone for Rc<T> {
    fn clone(&mut self) -> Self {
         unsafe { &mut *self.inner}.count += 1 
         Rc {
            inner: self.inner,
         }
    }
}
```

You may ask "is is safe to access like this" ?

```rust
 let cnt = &mut unsafe { &mut *self.inner}.count += 1 
```

This is safe because we know that there are no other threads that is accessing `inner` at all, Rc is not allowed to leave the thread, thats the same reason it can just be a standard `usize` operation and no need for atomic types, this is the implicit assumption that must not be violated , we know that there is no concurrent execution and no race might happen trying to access `cnt`, that is the reference count.
The whole functionality of Rc relies on the fact that it is not `Send`.
Btw `*self.inner` is a raw pointer

Take a look at this example

### Example 1

Here we have the foo method , its type signature constrains its parameter to be `Send` and then we pass in Rc<T> which does not implement `Send` to it

```rust
    fn foo<T: Send>(_: T) {}

    fn bar(x: Rc<()>) {
        foo(x)
    }
```

it appears within Rc and the traits is not implemented , we get an error

```rust
error[E0277]: `*mut Inner<()>` cannot be sent between threads safely
 --> src/main.rs:9:9
  |
9 |     foo(x)
  |     --- ^ `*mut Inner<()>` cannot be sent between threads safely
  |     |
  |     required by a bound introduced by this call
  |
  = help: within `Rc<()>`, the trait `Send` is not implemented for `*mut Inner<()>`
note: required because it appears within the type `Rc<()>`
 --> src/main.rs:1:8
  |
1 | struct Rc<T> {
  |        ^^
note: required by a bound in `foo`
 --> src/main.rs:6:11
  |
6 | fn foo<T: Send>(_: T) {}
  |           ^^^^ required by this bound in `foo`

For more information about this error, try `rustc --explain E0277`.
```

### Example 2

We create an Rc, clone it, then pass it into the thread.
```rust
fn main() {
    let x = Rc::new(1);
    let y = x.clone();

    std::thread::spawn(move || {
        drop(y); 
    });
    drop(x);
}
```

Then we get this error

```rust
`*mut Inner<i32>` cannot be sent between threads safely
   --> src/main.rs:60:24
    |
60  |       std::thread::spawn(move || {
    |       ------------------ ^------
    |       |                  |
    |  _____|__________________within this `{closure@src/main.rs:60:24: 60:31}`
    | |     |
    | |     required by a bound introduced by this call
61  | |         drop(y); //anything the thing in the closure has to be Send 
62  | |     });
    | |_____^ `*mut Inner<i32>` cannot be sent between threads safely
    |
    = help: within `{closure@src/main.rs:60:24: 60:31}`, the trait `Send` is not implemented for `*mut Inner<i32>`
note: required because it appears within the type `Rc<i32>`
   --> src/main.rs:1:8
    |
1   | struct Rc<T> {
    |        ^^
note: required because it's used within this closure
   --> src/main.rs:60:24
    |
60  |     std::thread::spawn(move || {
    |                        ^^^^^^^
note: required by a bound in `spawn`
   --> /Users/me/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/std/src/thread/mod.rs:727:8
    |
724 | pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    |        ----- required by a bound in this function
...
727 |     F: Send + 'static,
    |        ^^^^ required by this bound in `spawn`
```
now, we get an error because anything that is used in the async closure created by `std::thread::spwan` has to be `Send` and Rc is not Send 

## Sync

note :

- &T - shared reference

- &mut T - mutable reference
- Sync is almost defined in terms of `Send`

- if you have a type, where a reference to that type is allowed to be sent between threads then it is `Sync`, even if the type itself cannot be passed to some other thread. According to the documentation, "a type `T` is `Sync`, if and only if `&T` is `Send`"

- This is the reason Rc is not `Sync` too, because if it was `Sync` we can take a reference of `Rc` send it to another thread, clone it and do what we want with it, but this should not be, as the clone underlying impl requires that access happens on one thread

- But a MutexGuard is not Send but it is Sync , - they can't drop it, its behind a shared reference, they don't get any type of owned access, all they can do is read from it <https://github.com/tokio-rs/tokio/blob/master/tokio/src/sync/rwlock.rs#L146>

- AtomicPtr is just a raw pointer that impls Send

- if you create a sender receiver channel, whre you move types not between threads, theres not requirement that the types are Send and Sync

<https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html>

this is the type signature for the send method on Sender

``` rust
impl<T> Sender<T> 

pub fn send(&self, t: T) -> Result<(), SendError<T>>
```

- futures are Send and Sync for the prupose of multithreadded execution , you should see something like this

```rust
fn (_: Box<dyn std::future::Future<Output = ()> + Send>) {} 
```

### Can we implement Negative traits bounds for Send or Sync ?

- No we can't in the stable compiler, yet, lets say we wanted to implement a dummy MutexGuard with a !Send trait

```rust
struct MutexGuard<'a, T> {
    i: &'a mut T,
}

impl<T> !Send for MutexGuard<'_,T> {}

```

we get this error:

```rust
error[E0658]: negative trait bounds are not yet fully implemented; use marker types for now
  --> src/main.rs:16:9
   |
16 | impl<T> !Send for MutexGuard<'_,T> {}
   |         ^^^^^
   |
   = note: see issue #68318 <https://github.com/rust-lang/rust/issues/68318> for more information
```

It tells us to use marker types, and what we can use is `PhantomData` we're not actually storing an Rc, just pretend the type is there, so since, but more importantly it propagates its type, so because Rc is not Send and Sync , our MutexGuard will also be not Send and not Sync, heres the new code

```rust

struct MutexGuard<'a, T> {
    i: &'a mut T,
    _not_send: std::marker::PhantomData<Rc<std::rc::Rc<()>>
}

```

Obviously useful references :
- https://doc.rust-lang.org/std/marker/trait.Sync.html
- https://doc.rust-lang.org/std/marker/trait.Send.html 
- https://doc.rust-lang.org/nomicon/send-and-sync.html 


## Additional notes with timestamps i saw in a youtube comment, talks about `Cell` and stuff i omitted


```rust
@CPTSLEARNER
10 months ago (edited)
11:10 T does not have to be Clone in Rc<T>
11:45 If &mut is omitted, the code would still work, as dereferencing a mutable raw pointer (self.inner: *mut Inner<T>) gives mutable access to Inner<T>
13:10 &unsafe { &*self.inner }.value: &* dereferences the raw pointer and casts the Inner<T> to a shared reference, & casts the value to a shared reference
25:30 MutexGuard is Sync + !Send, Rc is !Send (clone Rc and send to another thread, reference count is not atomic) + !Sync (send &Rc to another thread and call clone, requires all access happens on one thread)
28:40 Cell is Send + !Sync, can't get reference to Cell in another thread, therefore safe to mutate in current thread as no other reference is mutating it. T must also implement Send + !Sync.
31:00 Application of Cell in graph traversal (can't take exclusive references, could walk same node), Cell allows mutation through a shared reference
36:20 If &mut T is Send, then T must be Send (std::mem::replace)
45:00 T is Sync because all the Arc instances reference T, T is Send because the last Arc must drop the inner type
46:00 &T is Send if T is Sync
47:30? Sender is !Sync, multiple shared references to Sender but only one in each thread
54:30 dyn syntax allows only one trait, exceptions are auto traits (Send, Sync)
59:50 thread::spawn requires type is 'static & and Send, not Sync as it doesn't take references
1:00:40 thread::scope does not need 'static & arguments, current thread can't return until scoped thread joined
```