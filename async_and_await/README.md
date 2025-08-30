## What does the async keyword do ?

Take a look at this code 

```rust 
async fn foo() -> usize {
    0
}
```

```rust 
fn foo2() -> impl Future<Output = usize> {
    async { 0 }
}
```


A value thats not ready yet but will eventually be a usize , its a promise that i will give you a usize 

 Don't run the following instructions until it reolves into its output type

`A Future does nothing until it is awaited `

This code will not print anything because it is not awaited 

```rust 
async fn foo1() -> usize {
    println!("foo")
}

fn main () {
    let _ = foo1()
}
```

Think of futures as exectuing in chunks , while its not ready it lets other things run , it runs until it can't make progress anymore, then it yields. 

Look at this , here we have one thread for handling terminal reads and one thread for a handling connections, but it gets worse if we have one thread for every operation we have to do.

If one connection is full you have to be able to write to the other.


```rust 
let read_from_terminal = std::thread::spawn(move|| {
    let mut x = std::io::Stdin(std::io::stdin());
    for line in x.lines() {
        //do something on user input
    }
})
```

```rust
let read_from_network = std::thread::spawn(move|| {
    let mut x = std::net::TcpListener::bind("0,0,0,0:8080").unwrap();
    while let Ok(stream) = x.accept() {
        //do something on stream
    }
})
```

## select! macro
(was getting tired in this section lol, slept off too)

- Waits on multiple futures and tells you which one finishes first, yield until something happens 

- Uses the idea of cooperative scheduling - if i don't run i let whoever is above me decide who runs next, and it might not be me 

- Cancellation tokens to cancel any future 

- Another name proposed for `select!` was `race!`

- The way cancellation works is that you describe the circumstances under which you cancel the operation 

- Your program can be in an intermediate state it only affects selects, this is an error case you should no 

- Fuse means its safe to await this future een though the future has completed in the past 

- `select!` keeps a bitmask for all available branches of execution and when it receives 

```rust 
let mut f1 = tokio::fs::File::open("foo");
let mut f2 = tokio::fs::File::open("bar");
let copy = tokio::io::copy(&mut f1, &mut f2);

fn foo2(cancel:tokio::sync::mpsc::Receiver<()>) -> impl Future<Output = usize> {
    async {
        println!("foo1");
        read_to_string("file1").await;
        println!("foo1");
        loop {
            select! {
                done <- read_to_string("file2").await => {
                    //continue or fall through to println below
                }
                cancel <- cancel.await => {
                    return 0
                }
                foo <- (&mut foo).await => {

                }
                _ <- copy.await 
            }
        }
        println!("foo1");
        let x = /*waiting on */ read_to_string("file2").await;
        println!("foo1");
    }
}

```

- If you're in an async context and you need to run something that you know might block for a long time , you can use `tokio::task::spawn_blocking` , i'm about to block allow other tasks to run too

```rust
tokio::task::spawn_blocking(async move ||{
    //computationally heavy task 
}).await
```

## join! macro 

- run all the operations concurrently and give me the output when they are all completed 

```rust 
let files: Vec<_> = (0..3)
    .map(|i| tokio::fs::read_to_string(format!("file{}", i)))
    .collect();

//compare
let file1 = files[0].await;
let file2 = files[1].await;
let file3 = files[2].await;

//to this 

let (file, file2, file3) = join!(files[0], files[1], files[2]);
```

- Only good if you have a few things, you don't want a tuple with 100 items lol 

- `joinall` and `tryjoinall` uses futures under the hood, `join` and `select` allows things to run concurrently, but they don;t allow them to run in parallel

- what does things is that you give it a future and it moves that future to the executor, Note that this is NOT a `thread::spawn`

- requires that you future you pass in is `Send + static` so it can be sent to other threads and because it does not know the lifetime of the runtime.

- You need to communciate the futures that need to run in parallel to the executor 

- Sometimes when people start using async await and they see that the performamnce drops its because they're not spawning anything and it runs on one thread, i mean ,of course, nothing gets to run in parallel 

- Spawn will stick the future into the executor and keep a pointer to that future 

- Futures assigned on the stack need to bre pinned so they cannot be moved

- Once you've started awaiting a future you can't move it unless its unpinned 

## async trait 

(i'm awake now)

We cant use async fns in traits, we don't know what the size 

``` rust 
struct Request;
struct Response;

trait Service {
    // fn call(_:Request) -> impl Future<Output = Response>;
    async fn call(&mut self, _:Request) -> Response>;
}

struct X;

impl Service for X {
    async fn call(&mut self, _: Request) {
        Response
    }
}

struct Y;

impl Service for Y {
    async fn call(&mut self, _:Request) {
        let z = [0, 1024];
        tokio::time::sleep(100).await;
        drop(z);
        Response
    }
}

struct FooCall<F>(F); 

fn foo(x: &mut dyn Service) {
    let fut = x.call(Request);
} // the size of fut depends on the stack variables that was used in the async block at theat point 
// so the compiler does not know the size of fut 

```

There really isnt a "good" day to deal with async fn calls so far 

The type of the thing it produces isn't knwon anywhere

we can fix this by using `#[async_trait]` attribute macro, async_trait rewrites the methods in the trait and implementation from 

```rust 
#[async_trait]
trait Service {
    async fn call(&mut self,_:Request) -> Response>;
}

#[async_trait]
impl Service for X {
    async fn call(&mut self, _: Request) -> Response {
        Response
    }
}

```

to this 

```rust 
trait Service {
    fn call(&mut self,_:Request) -> Pin<Box<dyn Future<Output = Response>>>;
}


impl Service for X {
    async fn call(&mut self, _: Request) {
       Box::pin(async move {
            Response
       })
    }
}

```
Heres the Cargo expanded version of our code where we use the async_trait attribute macro 

```rust 

#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::future::Future;
use async_trait::async_trait;
struct Request;
struct Response;
trait Service {
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn call<'life0, 'async_trait>(
        &'life0 mut self,
        __arg1: Request,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Response,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;
}
struct X;
impl Service for X {
    #[allow(
        elided_named_lifetimes,
        clippy::async_yields_async,
        clippy::diverging_sub_expression,
        clippy::let_unit_value,
        clippy::needless_arbitrary_self_type,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn call<'life0, 'async_trait>(
        &'life0 mut self,
        __arg1: Request,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = Response,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<
                Response,
            > {
                #[allow(unreachable_code)] return __ret;
            }
            let mut __self = self;
            let __arg1 = __arg1;
            let __ret: Response = { Response };
            #[allow(unreachable_code)] __ret
        })
    }
}
fn main() {}
```

Now, if i take the expanded form in the Service traits and place it in my code base and compile, it won't complain because it now has a well known size as it is a Heap allocated dynamically dispatched future (wow thats a mouthful)

The problem is you're now allocating all your futures in the heap, now imagine if you do a read, lets rename our service trait to `AsyncRead`, now, everytime we do a read we are doing a heap allocation and extra pointer indirection, it might no well if you have to use it at the bottom of your stack, its better for higher level stuff

```rust 
    trait AsyncRead {
         fn read(&mut self, _:file) -> Pin<Box<dyn Future<Output = Response>>>;
    }
```

Another method of implementing async methods in traits, we can do is an associated type, rust will also know how large the associated type is and we can communicate it to callers 

```rust
trait Service {
    type CallFuture = Future<Output = Response>;
    fn call(&mut self, _:Request) -> Self::CallFuture;
}


impl Service for X {
    type CallFuture = Pin<Box<Future<Output = Response>>>;
    fn call(&mut self, _: Request) -> Self::CallFuture {
       async { Response }
    }
}

```

## Which one to use ? std::sync::Mutex or tokio::sync::Mutex?

```rust
async fn main() {
    let x = Arc::new(Mutex;:new(0));
    let x1 = Arc::clone(&x);
    tokio::spawn(async move {
        loop {
            let x = x1.lock();
            tokio::fs::read_to_string("file").await;
            *x1.lock() += 1;
        }
    });
    let x2 = Arc::clone(&x);
    tokio::spawn(async move {
        loop {
            *x2.lock() += 1;
        }
    });
}
```

Suppose we are in a situation we execute `x1`, it locks and reads the string, the lock is still held by `x1`, now when `x2` tries to enter its own branch of execution and acquire the lock, it can't because its still held by `x1` so it just blocks the whole thread , it knows nothing about asynchrony, that means the executors thread is blocked, then it never continues reading the string, then the first future never drops its lock on, so the lock is never released and the lock on l2 never completes and we effectively have a deadlock, this is why they avoid

Now the difference is that in async aware locks, when it fails to take the lock it will yield rather that blocking the thread (VERY IMPORTANT)

This is why we need async aware mutexes - but the downside is that they are slower (theres more heavy lifting behind the scenes)

In general we want to use `std::sync::mutex` standard library mutexes as long as your critical section (part in a concurrent program where we access a shared resource) is short and does not have any .awaits (unlike our `read_to_string`)

So if we didn't have our `read_to_string` method call and we just increment or decemrenting the number, then the `std mutex` is ok 

```rust
async fn main() {
    let x = Arc::new(Mutex;:new(0));
    let x1 = Arc::clone(&x);
    tokio::spawn(async move {
        loop {
            *x1.lock() += 1;
        }
    });
    let x2 = Arc::clone(&x);
    tokio::spawn(async move {
        loop {
            *x2.lock() += 1;
        }
    });
}
```

## Smol rant 

tbh the syntax  `*x.lock() += 1` is a bit confusing, something like

```rust 
x.lock()
num += 1 // don't think about where the num comes from :-)
```

would be clearer, maybe its because i'm still thinking of the java syntax (i did java lolll)

Okay just checked the docs the even though we initialize the `Mutex` with a 0 , we first access the lock, unwrap the value and modify it, so we have something like this 


```rust
async fn main() {
    let x = Arc::new(Mutex;:new(0));
    let x1 = Arc::clone(&x);
    tokio::spawn(async move {
        loop {
             let num = x1.lock.unwrap()
             num += 1;
        }
    });
}
```

## Difference between tokio::spawn and thread::spawn

- `tokio::spawn` gives the future it has passed to the executor, for the executor to run concurrently with other futures.

- A `thread::spawn` spawns an OS thread that will run in parallel with everything in your program - its also does not take a future it takes a closure , if you wanted to await a future in a `thread::spawn` you would have to create your own executor.

- If you use `tokio::spawn` you have to have yield points so that you would not block the executor and let the other futures run .

- In `thread::spawn` thats not a problem because the OS can preemptively interrupt you, they are not cooperatively scheduled 

- When a future fields, it yields to whatever awaited it.


## The what, how and why of async await 

Note - 
This is the 4hr and 10 min video where jon gjengset goes deep into async await mechanics in rust 

Lets start with an example where i'm trying to connect to a server

```rust 
let x = TcpStream::connect("127.0.0.1");
    let y = TcpStream::connect("127.0.0.1");
    x.write("foobar");
    y.write("foobar");
    assert_eq!(x.read(), "barfoo");
    assert_eq!(x.read(), "barfoo");

    let fut_x = TcpStream::connect("127.0.0.1")
        .and_then(|c| c.write("foobar"))
        .and_then(|c| c.read())
        .and_then(|(c,b)| b == "barfoo");
    println!("{:?}, fut");

    let fut_y = TcpStream::connect("127.0.0.1")
        .and_then(|c| c.write("foobar"))
        .and_then(|c| c.read())
        .and_then(|(c,b)| b == "barfoo");
    println!("{:?}, fut");

    let a: Executor;
    let x = a.run(fut_x);
    let y = a.run(fut_y);
```

An executor is something that you give futures to and it makes sure that they get done

spawn tells the executor just run this thing and make sure it gets run at some point


what does the executor do ?? we're about to find out 

Okay so this video was made 6 years ago and the trait definition of futures have changed since then

This is it in the video, the old trait impl
```rust
pub trait Future {
    type Item;
    type Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error>

    fn wait(self) -> Poll<Self::Item, Self::Error>
    where 
        Self: Sized,
    { ... }
}
```

Here we have two types representing both the item that the future will return and the error it might throw separately 

But the latest version has replaced it with one type `Output` 

```rust 
pub trait Future {
    type Output;

    // Required method
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```
but depending on the trait impl, we can still assign a `Result<T, U>` type with the item it returns and an error to the Output , for example

```rust 
impl<R, W> Future for CopyBuf<'_, R, W>
where
    R: AsyncBufRead,
    W: AsyncWrite + Unpin + ?Sized,
Source
type Output = Result<u64, Error>
```

this decision was to probably make it more "generic" for more trait impl that can't just be expressed with an Error and an Item, some don;t even have error in their trait impl
