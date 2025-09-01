

## WHy not just use a primitive types 

When we use types like Bool and Number there are only safe ways to interact withthem acorross threads to avoid data races 

Data races are undefined behaviour 

It makes sense to have different API becuase they are different data types 

we are issuing different instructions to the CPU under the hood 

There is no guarantee that a value written will be seen by another thread, so thhis is where atomics comes into play 

in the video jon said rust does not have a memory model as he reads from the docs but now its still not complete https://doc.rust-lang.org/reference/memory-model.html

Rust generally follow the C11 memory model and in this tutorial we are goint to be following the memory model of C++ we are going to use them because they have good docs on the explanation of what each atomic means in the memory model 


## Atomicusize 

https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html 

You keep Atomic types on the heap like Box or an Arc because that would allow us to share a reference 

You an operate on an AtomicUsize using shared references to `self` that is `&self` , the reason is that the compiler geneates multiple CPU instructions that make it safe for multiples threads to access it at the same time

```rust 
//self is safe to be accessed by a shared reference here 
pub fn load(&self, order: Ordering) -> usize
```

methods

load and store load out the value stored in the usiz e
store stores it 

swap - does both 

Ordering - which set of guarantees that is expected for this memory access

compare and swap, compare_exchange, compare_exchange_weak -> ways of reading a value and swapping it out doing so conditionally and in one atomic step reading and swapping out its value in one atomic step

When you do a load and a store another thread can come in between your operations (and maybe modify values)
wi9th compare_and_swap no value can come inbetween 

fetch methods -> they try to avoid something happening between loading and storing or modfying a value or reading 
eg fetch_add will load a value and add a value to it 

do u64s on x86 systems have atomic access -> on some platforms some normal types hape additional guarantees , like intel x86 any access to au64 value is automatically assuming it does not cross a cache line boundary?

### Why is atomics not exhaustive ?

No code is ever allowed to assume that 

```rust 
pub enum Ordering {
    Relaxed,
    Release,
    Acquire,
    AcqRel,
    SeqCst
}
```
will ever the Ordering that exists , rust is planning to add ConsumeOrdering , if we want the ability to add stuff later it needs to be non exhaustive 

### Are the memory ordering types related to the memory models architectures like x86 or ARM 

They are not related to the architexture -> they are related to guarantees that operation will give you, how different architectures implement those guarantees vary from arch to arch 

### Difference between atomic and mutex
Atomis do not have locking -> different threads can operate on the same value at the same time in some well defined way 

One thread gets to access a value at the time while other threads waut, the mutex guards a larger part of the code no other thread is allo0wed to execute this section of code while i am accessing it 

### Do atomic operations not block other threads from accessing 
No atomic operations are lock free, some certain archs that dont have fetch_add can use a modification of compare_and_swap and they might have to wait for threads but its not necessarily  "locking"cuz theres no mutex 

The atomic opeations are not just per CPU in the sense that they just modify the CPU instructions they also change the compiler semantics , they limit both what the CPU and compiler can do about a given memory access

So even if you're on intel x86 you will want to use atomic operations for compiler guarantees 


## Atomic operations and Unsafe Cell
store swap and friends?? all take an immut reference to self and not mut self because they rely on unsafe cell 

All the atomic instructions use an unsafe cell where get is called on to get a pointer to the value and call assembly instructions on that 

https://doc.rust-lang.org/src/core/sync/atomic.rs.html#2605 

mistakenky came across this crate lol
https://docs.rs/atomic/latest/src/atomic/lib.rs.html#66

So an atomic usize is a UnsafeCell usize that uses special instructions to access the underlying value 


Unsafecell is fundamentally the only way to get mut access to a shared ref 

## Atomics are shared via Arc and not Box but the only reason is that Arc is Send + Sync while Box is not so it smore convenient ?

The reason why Arc is used over Box is because , when you create a boxed value it gets stored on the heap, so you have an owned value

But if you spawn two threads, the threads require that the closure you pass in has a `'static` lifetime, if a reference to the Box is passed to both threads, then the reference does not have a static lifetime it has the stack lifetime of the Box
https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html 


But when using you can clone the Arc and give each threads their own individually Owned and static Arc , then you can pass in the usize 

You can use `Box::leak` , itn leaks the value on the heap which will give you back a static reference 
because it will never call the destructor

(That is we are converting a `Box<T>` to a `'static mut T`)

Also note that we add the type signature as `&'static` as a way to cast it from the mut reference returned from `Box::leak`

```rust
let l : &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
```

https://softwaremill.com/leaking-memory-on-purpose-in-rust/ 
https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak 

## What can we use them for 

### Implementing a Mutex 
For this example we are going to implement a `Mutex<T>` type 

A mutex is basically a combination of an atomic boolean to signify whether the lock is being held, and an UnsafeCell to be able to give out a mutable reference to the item locked behind the Mutex 

```rust
pub struct Mutex<T> {
    locked: AtomicBool,
    V: UnsafeCell<T>
}
```

for this we are going to use spinlocks, basically spins until it can access the value behind the lock 

Dont use spinlocks , most of the time you do not want to use them , also most of the time you do not want to implement your own Mutex 

https://matklad.github.io/2020/01/02/spinlocks-considered-harmful.html 

Reasonable excerpt from article 
```md
Because spin locks are so simple and fast, it seems to be a good idea to use them for short-lived critical sections. For example, if you only need to increment a couple of integers, should you really bother with complicated syscalls? In the worst case, the other thread will spin just for a couple of iterations…

Unfortunately, this logic is flawed! A thread can be preempted at any time, including during a short critical section. If it is preempted, that means that all other threads will need to spin until the original thread gets its share of CPU again. And, because a spinning thread looks like a good, busy thread to the OS, the other threads will spin until they exhaust their quants, preventing the unlucky thread from getting back on the processor! 
``

//Also priority inversion issues 
https://en.wikipedia.org/wiki/Priority_inversion

## Designing the mutex 

We have a simple impl of the struct , `new` here lets us instantiate a new Mutex type with the default locked state being `UNLOCKED = true` that returns a Self that holds both the boolean and the mutable reference v to the underlying T via UnsafeCell

```
impl<T> Mutex<T> {
    pub fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(UNLOCKED),
            v: UnsafeCell::new(t),
        }
    }
    fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {

    }
}

```

now for the with_lock method , we are going to start with a (horrible) naive implementation for testing sake 

```rust 
 pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        //naive version of the logic 
        while self.locked.load(Ordering::Relaxed) != UNLOCKED {}
        self.locked.store(LOCKED, Ordering::Relaxed);
        //Safety: we hold the lock, therefore we can create a mutable refernce 
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed)
        ret 
    }
```
The logic is simple , **explain logic 


## Testing our mutex
We'll first create our Mutex value, then we'll spin up 10 threads, each thread is going to increment the value a 100 times, so the final value of the value in the Mutex should be 1000

```rust 
fn main() {
    //remember wehy we are using this Box::leak turns a Box<T> into a 'static mut T, so with that we can pass our l value into the thread closure since it has a static lifetime 

    let l: &'static _ = Box::leak(Box::new(Mutex::new(0)));

    let handles: Vec<_> = (0..10)
        .map(|_| {
            spawn(move || {
                for _ in 0..100 {
                    l.with_lock(|v| {
                        *v += 1
                    })
                }
            })
        }).collect();
    for handle in handles {
        handle.join().unwrap()
    }
}
```

the first error we encounter is that we cant compile the code because we get an error that

```rust 
`UnsafeCell<i32>` cannot be shared between threads safely
  --> src/main.rs:44:19
   |
44 |               spawn(move || {
   |  _____________-----_^
   | |             |
   | |             required by a bound introduced by this call
45 | |                 for _ in 0..100 {
46 | |                     l.with_lock(|v| {
47 | |                         *v += 1
...  |
50 | |             })
   | |_____________^ `UnsafeCell<i32>` cannot be shared between threads safely
   |
   = help: within `Mutex<i32>`, the trait `Sync` is not implemented for `UnsafeCell<i32>`
note: required because it appears within the type `Mutex<i32>`
  --> src/main.rs:8:12
   |
8  | pub struct Mutex<T> {
   |            ^^^^^
   = note: required for `&Mutex<i32>` to implement `Send`
note: required because it's used within this closure
  --> src/main.rs:44:19
   |
44 |             spawn(move || {
   |                   ^^^^^^^
note: required by a bound in `spawn`
  --> /rustc/6b00bc3880198600130e1cf62b8f8a93494488cc/library/std/src/thread/mod.rs:723:1

For more information about this error, try `rustc --explain E0277`.
```

Our Mutex needs to implement Sync for it to be able to be accessed from multiple threads , and the type needs to impl Send for it to be able to be shared across multiple threads (also needs to impl Sync because the lock might access the value )

```rust 
unsafe impl<T> Sync for Mutex<T> where T : Send {} 
```

After the impl the code compiles, we then add an assertion to check if the values get updated correctly across the multiple threads, if so then then total number should be 1000

```rust
assert_eq!(l.with_lock(|v| *v), 10 * 100);
```
The code works, but there are actually problems that are hard to reproduce , lets go back to our `with_lock` method 

```rust 
  pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        //we're doing a load
        while self.locked.load(Ordering::Relaxed) != UNLOCKED {}

        //in between both load and store operations, another thread might run and modify or read a value, which we do not want so thats why we need compare and swap

        //then we are doing a store 
        self.locked.store(LOCKED, Ordering::Relaxed);


        //Safety: we hold the lock, therefore we can create a mutable refernce 
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed)
        ret 
    }
```

Its possible for a scenario to be that both threads see that the mutex is unlocked, so they exit the while loop , lock the mutex, both get a mutable reference to the underlying value and modify it (which is undefined behaviour) , then unlock the mutex, the speed of the computer make these scenarios seem impossible to happen.

Now lets add a `thread::yield_now()` operation in between the load and the store operations to demonstrate that this is possible.

```rust 
  pub fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        //we're doing a load
        while self.locked.load(Ordering::Relaxed) != UNLOCKED {}
        std::thread::yield_now();
        self.locked.store(LOCKED, Ordering::Relaxed);
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed)
        ret 
    }
```

Then our assertion fails , and panics, we didn't reach the final expected value 

```rust 
thread 'main' panicked at src/main.rs:58:5:
assertion `left == right` failed
  left: 996
 right: 1000
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

What happened here? Because the threads run at the same time concurrently, some increments are overwritten, giving us a wrong value, eventually we dont reach 1000, with the `yiled_now` we are modelling that some thread gets pre empted between the load and the store operations 

## Q -> doesnt the compiler compute the sum before hand
The compiler doesnt compute the sunm across thread boundaries, even though technically it can be computed statically 

## Would ThreadSanitizer catch this
Yeah, you have two threads writing to the same memory location (write-write conflict)

## Sleeping there would reduce lock contention and concurrency issues / data races?
You would have more lock contention if the critical section is shorter, sleeping reduces contention ## 38.06 How lol 

```rust 
  l.with_lock(|v| {
       *v += 1
  })
```


If you dont have multiple threads the need for Atomics is unlikely 

## Using compare_exchange
https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.compare_exchange 

We have to fix the race between the load and the store , we can do that using the `compare_exchange` atomic operation

compare_exchange is more powerful thatn acomparw_and_swap  -> it lets you specify the memory ordering for if the operation failed or suceeded 

```rust
pub fn compare_+exchange(
    &self,
    current:usize,
    new:usize,
    success: Ordering,
    failure:Ordering
) -> Result<usize,usize>
```

compare_excahnge is a single atomic operations that is both te read and write, what it basically does is, in this scneario, is "set the boolean value to true, if and only if the boolean value is false, and do it in such a way that no thread can modify its value in between"

Will return an error if the value was not updated, the value obtained in the Ok or Err will be the value it was at the time the operation happened (that is when the memory location was accessed)

compare_exchange is an expensive operation , one core is holding the lock that guards a value, mutiples cores are trying to get exclusive access (ownership) to the same memory location checking whether its the current value and so it gets passed between them

The MESI protocol is a great example of understanding how this works at the lower level , it deals with cache lines and coherence but here "memory location" will be used in place of those terms 

A location in memory can either be shared or exclusive , `compare_exchange` works on the idea of exclusive access, each core has to coordinate with other cores, or a location in memory can be marked as shared where multiples core can have a value in the shared state at the same time 

So if the value is in the shared state while the lock is being held , then the "ownership bouncing" is going to be avoided 

Back to our spinlock implementation, we are going to add an inner loop in the while loop which is commmon in spinlock implementations, we perform a read only access where if we fail to acquire the lock, we just read the value (enter the read only loop) , which is more efficient when you have high contention to avoid performing the expensive compare_exchange operation 

```rust 
 fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self.locked.compare_exchange(
            UNLOCKED, 
            LOCKED, 
            Ordering::Relaxed, 
            Ordering::Relaxed
        ).is_err() {
            while self.locked.load(Ordering::Relaxed) == LOCKED {}
        }

        //Safety: we hold the lock, therefore we can create a mutable refernce 
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);
        ret 
    }
```

Now the performance degaradation that we are talking about here seems close to insignificant that is in nanoseconds, and can only be observed if you actually plot a throughput graph or unless you are working with a value that has high contention.

### Is compare_exchange faster than locking a mutex 
The difference between both compare_exchange is that mutex waits -> your thread will be blocked until you acquire the lock, but compare_exchange does not, (but you can choose to do that, in this case we are spinning)  it goes ahead to perform the operation and its either it suceeded or failed, it does not wait, its more primitive, 


## Using compare_exchange_weak 
if the cur value did not match the val you passed in then compare_exchange will fail, but compare_exchange_weak is allowed to fail spuriously (anyhow), but why? It depends on the instructions the CPU supports.

Lets look at the INTEL and ARM architectures instructions to explain this

The INTEL x86 architecture supports CAS `compare_and_swap` operation that basically does what is says it CAS's

THE ARM architecture has the `LDREX` and `STREX` instructions -> THey are the `load_exclusive` and `store_exclusive` operations, the `LDREX`  takes exclusive access of a memory location then load a value to a memory location, while `STREX` storeS a value to a memory location ONLY if we have exclusive access to it (STREX is very cheap).

`compare_exchange` is implemented with a combination of the `LDREX` and `STREX` instructions in a loop for the compare_exchange semantics of "only fail if the current value stays the same", which would need to be implemented with a nested loop, and will generate more assembly code (less efficient) -> which is where

`compare_exchange_weak` comes in , can be impl with `LDREX` and `STREX` directly, (on INTEL archs the ix sequence is basically a CAS operation) 

Now with this information, we can deduce that if what we are doing requires us to do it in a loop, we should rightly use `compare_exchange_weak` so lets update our code


 ```rust 
 fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self.locked.compare_exchange_weak(
            UNLOCKED, 
            LOCKED, 
            Ordering::Relaxed, 
            Ordering::Relaxed
        ).is_err() {}

        //Safety: we hold the lock, therefore we can create a mutable refernce 
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);
        ret 
    }
 ```

If we want it to only fail if the current value changed then we use `compare_exchange` 

Also its good to know that, these instructions are also referred to as `Load link (LL) and Store-Conditional(SC)`

```md
Load-Link (LL) and Store-Conditional (SC) are a pair of instructions used in modern processor architectures, particularly RISC architectures, to implement atomic operations and synchronization primitives in a multithreaded environment.
```

compare_exchange_weak can fail for all sorts of problems but compare_exchange can only fail for one -> that is if the current value is not the same as the expected value 

Now going back to our code , lets add a `std::thread::yield_now()` to the body of the outer and inner loop, to try to cause a race condition in the logic 

`std::thread::yield` gives up execution for other threads to run and access some shared resource or value 

```rust 
 fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self.locked.compare_exchange_weak(
            UNLOCKED, 
            LOCKED, 
            Ordering::Relaxed, 
            Ordering::Relaxed
        ).is_err() {
            //MESI protocol: stay in S when locked 
            while self.locked.load(Ordering::Relaxed) == LOCKED {
                std::thread::yield_now();
            }
            std::thread::yield_now();
        }

        //Safety: we hold the lock, therefore we can create a mutable refernce 
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);
        ret 
    }
```

No error occurs , so that means our code is right? No it is not actually lets go into Irdering to figure out why 


## Ordering 
They dictate what is allowed to happen when you run some certain code that executes some atomic operation, lets make some simple test demonstrating relaxed orering 

```rust 
#[test]
fn too_relaxed() {
    use std::sync::atomic::AtomicUsize;
    let x: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));
    let y: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));

    let t1 = spawn(move || {
        let r1 = y.load(Ordering::Relaxed);
        x.store(r1, Ordering::Relaxed);
        r1
    });

    let t2 = spawn(move || {
        let r2 = x.load(Ordering::Relaxed);
        x.store(42, Ordering::Relaxed);
        r2

        let r1 = t1.join().unwrap();
        let r2 = t2.join().unwrap();
        //r1 == r2 == 42
    })
}
```


Memory orderings specify the way atomic operations synchronize memory.

its possible for r2 to be 42 even though the store of 42 happens AFTER r2 is read 
- There are no guarantees what a thread can read after a thread wrote under Ordering::relaxed
- Theres no guarantee what happens after or before some memory ordering eg  a value is read after its written, or its written afer its read 

To explain this better lets introduce the concept of a modification order, each atomic has a modification order that would be repreeented bya function MO()
MO(x): 0 42 
MO(y): 0 42

the value of 0 are initially stored into x and y, then 42 is stored in x 

In the x.load(Ordering::Relaxed), its possible for the value that x loads to be 42 because it exists in the Modification Set of x , the Order doesnt matter.

Yes, even though the load comes before the store in the code, the CPU is allowed to execute them out of order, but why would it do that? Well for performance gains 

According to the rust docs it says under Ordering::Relaxed "No ordering constraints, just atomic operations"

Theres also the topic of speculative executuon , in our code that we have theres not a reason to do so , because there are no branches but assuming we introduced an if statement like so 

```rust
let t2 = spawn(move || {
    let r1 = y.load(Ordering::relaxed);
    if z == 3 {
        y.store(42, Ordering::relaxed);
    }
    r2
});
```

Then the CPU will speculaitvely execute the code inside the if statement body regardless of the value, then store the value if it matches the condition 

Memory Ordering is not just about what the CPU is allowed to do , but the compiler too 

Now lets go back to our `with_lock` method of our mutex , if we perform operations such as this, guess what? it is actually valid because of the memory ordering , but the problem is that it will violate exclusivity , that is one thread modifying the value at a time and also the order in which the unsafe operations happen


```rust 
 fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        *unsafe { &mut *self.v.get() } += 1;
        while self.locked.compare_exchange_weak(
            UNLOCKED, 
            LOCKED, 
            Ordering::Relaxed, 
            Ordering::Relaxed
        ).is_err() {
            //MESI protocol: stay in S when locked 
            while self.locked.load(Ordering::Relaxed) == LOCKED {}
        }
        //Safety: we hold the lock, therefore we can create a mutable refernce 
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Relaxed);
        ret 
    }
```

Relaxed is too weak, we have to use another type of Ordering the next ones after it in the docs is release and acquire which sound like the operations performed with locks (you release a lock so that the shared resource can be accessed , you acquire a lock to exclusively write/modify a shared resource)

## Ordering::Release 

If we do the store with release, any load of the same value that uses the ACQUIRE Ordering, must see all operations that happened before the store as having happened before the store, no reads or writes in the current thread can be reordered after this store

The rust docs says "When Ordering::Release gets coupled with a store, all previous operations become ordered before any load of this value with Acquire (or Stronger) Ordering"

```rust
    fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self.locked.compare_exchange_weak(
            UNLOCKED, 
            LOCKED, 
            Ordering::Acquire,
            Ordering::Relaxed
        ).is_err() {
            while self.locked.load(Ordering::Relaxed) == LOCKED {
                std::thread::yield_now();
            }
            std::thread::yield_now();
        }

        //Safety: we hold the lock, therefore we can create a mutable refernce 
        let ret = f(unsafe { &mut *self.v.get() });
        self.locked.store(UNLOCKED, Ordering::Release);
        ret 
    }
```
The load that happens in the compare_exchange in the code above, if it uses Acquire Ordering, must see any operations that we did in the line of code with the `Ordering::Release` below 

If we keep the 

```rust 
self.locked.store(UNLOCKED, Ordering::Relaxed)
```

 the problem is that there's is no guarantee that the thread that acquires the lock will see the changes that occurred in the line above it, that is, this line 

 ```rust 
 let ret = f(unsafe { &mut *self.v.get() });
```

But with Acquire - Release Ordering we have two guarantees 

- That `let ret = f(unsafe { &mut *self.v.get() })`; can never be re-ordered after `self.locked.store(UNLOCKED, Ordering::Relaxed)`

- And that anything we do in `ret` must be visible after an Acquire load of the same value 

Acquire-Release establish a relationship between what happens before what , that is in our Mutex scenario between the thread that releases the lock, and the thread that takes the lock -> it establishes an ordering

Its kinda like an invariant that says "anything that happened before the operation that did the store, happened after the operation the did the load"

## Ordering::AcqRel
When we pass it into an operation that has a read and a write it says "Do the load with acquire semantics and do the store with release semantics"

AcqRel is mostly used when you are doing a single modification operation , like fetch_add, theres no critical section that we want to be synchronized with other threads 

```rust
    fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self.locked.compare_exchange_weak(
            UNLOCKED, 
            LOCKED, 
            Ordering::AcqRel,
            Ordering::Relaxed
        ).is_err() {
     /// rest of the code 
    }
}
```

Here in our code we changed the Ordering to AcqRel to show its usage, but we don't actually need it here , we have store operations and a criitical section, Acquire and Release semantics are okay 

### What is the second Ordering:: argument in the compare_exchange_weak for?
- What ordering should the load have if the load indicates that you shouldn't store 

- what is the ordering if you fail to take the lock? We can leave it as Ordering::relaxed , if we fail to take the lock we dont have to do some exclusive access or coordination among threads 

### Why 
Intel x86 guarantees acquire release semantics for all operations (so the ordering::relaxed doesnt matter or gets overridden??) yeah basically , its baked into the CPU architecture itself 

Architectures like ARM do not guarantee that, it will give you relaxed ordering semantics if asked and this show the trickiness of testing concurrent code and its gotchas 

use relaxed when instruction order doesnt matter for all the threads eg a generic counter across threads 

## Fetch Operations 
https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.fetch_add

We tell the CPU how to compute the new value (with Ordering) instead of telling it what it will be, these operations are called fetch_add or fetch_sub because it tells you what the value was when it incremented (or decremented it) 

We cant actually use an atomic add and load to mimic this behaviour which shows the usefulness of fetch operatiosn because in between the atomic operation ( eg add ) and loading the value another thread might modify the value, so we can never have a guarantee that that wouldnt happen

Its a single atomic operation 

fetch_update is an interesting opreation -> it takes in a closure , that given the current value, it should return the new value 

You can see the type signature of F yeah? 

```rust 
pub fn fetch_update<F>(
    &self,
    set_order: Ordering,
    fetch_order: Ordering,
    f: F,
) -> Result<usize, usize>
where
    F: FnMut(usize) -> Option<usize>,
```

So now think of fetch_add as a fetch_update that takes in a closure that increments the value by 1 

What makes fetch update reallly weird is that for the other fetch operations they are natively implemented in the CPU , but fetch_update is really just a compare_exchange loop 

sauce https://doc.rust-lang.org/src/core/sync/atomic.rs.html#2017 

```rust 
    pub fn fetch_update<F>(
        &self,
        set_order: Ordering,
        fetch_order: Ordering,
        mut f: F,
    ) -> Result<*mut T, *mut T>
    where
        F: FnMut(*mut T) -> Option<*mut T>,
    {
        let mut prev = self.load(fetch_order);
        while let Some(next) = f(prev) {
            match self.compare_exchange_weak(prev, next, set_order, fetch_order) {
                x @ Ok(_) => return x,
                Err(next_prev) => prev = next_prev,
            }
        }
        Err(prev)
    }
```

Also notice that they correctly use compare_exchange_weak as its in a loop , so thats the main difference fetch_update and fetch_add , first is a CE loop , second is a single atomic CPU instruction 

Now going back to the [Portability](https://doc.rust-lang.org/std/sync/atomic/#portability) section under the rust atomic page, it says  

```md 
Atomic types and operations are not guaranteed to be wait-free. This means that operations like fetch_or may be implemented with a compare-and-swap loop
```

If fetch_add doesn't exist on your arch it'll just be impl with a compare_and_exchange loop 

## If some things are impl under the hood with compare_exchange then why doesnt they all return a Result 
- Fetch methods eg fetch_add ALWAYS succeeds 

## Sequentially Consistent Ordering SeqCst 

1.40.0 -> 

But but why? why did the programmers and designers even come up withmemory ordering, the re arranging of concurrent operations and all? 

Its actually okay in a mutex and it causes no problems 

Everything in rust by default has an ordering, now for the others (Semantics) if you want stronger guarantees you have to opt into them 

How does seq consistent ordering change this 

## SeqCst 
Like Acq Rel with the additional guarantee that all threads see all sequentially consistent operations in the same order 
```md
    What are the possible values for z ? 
    is 0 possible? if we make is SeqCst , 0 is not possible 
    Restrictions: 
    we know that t1 must run "after" tx
    we know that t2 must run "after" ty
    Given that..
    .. tx .. t1 ..
        ty t2 tx t1 -> t1 will incr z
        ty tx ty t2 t1 -> t1 will incr z
    .. tx .. t1 .. t1 ty t2 -> t2 will incr z
    Seems impossible to have a thread schedule where z == 0 

            t2    t1
                   v
    M0(x): false true
    
            t1
    M0(y): false true
    - Is 1 possible?
    Yes: tx, t1, ty, t2
    - Is 2 possible?
    Yes: tx, ty, t1, t2
```

Because we have set the Ordering to SeqCst in _tx, _ty t1 and t2 , after x and y have been set to true in 
_tx and _ty , no thread in t1 and t2 is allowed to observe x and y ever being false , a happens before relationship has already been established
SeqCst operations are only in relation to other operations with Ordering::SeqCst, that is sequentially consistent ordering 
SeqCst has the strongest ordering 
SeqCst is AcquireRelease with the stronger guarantee that all SeqCst operations must be seen as happening in the same order on all threads 

## Testing 
-Hard for the human brain to model these things , code can fail to do what it does without panicking

ThreadSanitizerAlgorithm - every load , store and atomic ix gets special ixs added to them they get tracked and when 
two threads that writes to the same mem location 
two threads that write before or after one and theres no "happens before" relationship (Ordering) between them

https://github.com/google/sanitizers/wiki/threadsanitizeralgorithm 

For rust theres [loom](https://github.com/tokio-rs/loom), which implements a paper called [CDSChecker](http://demsky.eecs.uci.edu/publications/c11modelcheck.pdf)

Loom works by taking a concurrent program, instrumented it, giving it its own loom types for the Atomics, Ordering, eg `loom::sync::Atomic` `loom::sync::atomic::Ordering`

Keeps track of all the values that have stores, every execution 

All possible thread interleavings and memory orderings 

Running your loom tests -> use loom primitives 

`loom::model` is what runs the different permutations of possibilities , and in it you pass in the closure you want to test 


## SeqCst example is not really solid
Notice some papers that they impl Concurrent Data Structures and look at where they used memory ordering 

The name loom makes sense it spins threads in may ways -> commmenter 

You cant fully model something like `Ordering::Relaxed` even with loom too realxed , too many possibilities

It models `Acquire-Release` correctly still


compiler fence compiler is not allowed to move things above or below the fence , its mostly used for preventing threads from racing with themselves that mostly happens when you are using signal handlers 

Establishes a happen before relationship between two threads without talking about a memory location 

AtomicPtr specialized typee of Atomic Usize that operates on pointers and keeps pointer properties 

DONT WRITE LOCK FREE CODE UNLESS YOU REALLY HAVE TO! YOU CAN JUST USE A MUTEX 

Now, if you somehow need to use lock free programming ,
Use Loom, use Thread Sanitizer , use Miri  find a paper that implements the algorithm you are trying to implement, and follow it , ask others to Vet it for you, just make sureee you get it right 

AGAIN, DONT WRITE LOCK FREE CODE UNLESS YOU REALLY HAVE TO! YOU CAN JUST USE A MUTEX 

## Further reading 
https://en.cppreference.com/w/cpp/atomic/memory_order.html --> MOST IMPORTANT READ
https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak
https://people.cs.pitt.edu/~melhem/courses/2410p/ch5-4.pdf 
https://doc.rust-lang.org/std/thread/fn.yield_now.html 
https://www.cs.utexas.edu/~pingali/CS377P/2018sp/lectures/mesi.pdf
https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html#method.compare_exchange 
https://www.reddit.com/r/rust/comments/1kx024i/disappointment_of_the_day_compare_exchange_weak/
https://en.wikipedia.org/wiki/ABA_problem
https://medium.com/@levinet.nicolai/exclusive-monitors-ldrex-strex-ensuring-atomicity-456ea5908bdb
https://medium.com/@levinet.nicolai/ldrex-strex-in-action-implementing-mutexes-and-semaphores-38b41b1b35d6
https://math.stackexchange.com/questions/128778/is-equality-the-same-as-identity
https://preshing.com/20150402/you-can-do-any-kind-of-atomic-read-modify-write-operation/ 
https://www.reddit.com/r/rust/comments/11pgs04/trying_to_visualize_how_memory_ordering_and/
https://dictionary.cambridge.org/dictionary/english/spuriously 
https://en.wikipedia.org/wiki/Load-link/store-conditional


https://www.cs.cmu.edu/~guyb/papers/3503221.3508433.pdf
https://medium.com/@tylerneely/fear-and-loathing-in-lock-free-programming-7158b1cdd50c 
https://www.reddit.com/r/cpp/comments/vg4myt/is_lockfree_programming_is_always_better_than/ 
https://www.youtube.com/watch?v=ZQFzMfHIxng -> CppCon 2017: Fedor Pikus “C++ atomics, from basic to advanced. What do they really do?” 
http://demsky.eecs.uci.edu/publications/c11modelcheck.pdf -> Paper that Loom for rust was implemented with 
loom docs
https://github.com/google/sanitizers/wiki/threadsanitizeralgorithm


