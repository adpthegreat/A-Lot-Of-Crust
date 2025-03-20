


## What does the async keyword do ?

Take a look at this code 

```rust 
async fn foo() -. usize {
    0
}
```

```rust 
fn foo2() -> impl Future<Output = usize> {
    async { 0 }
}
```


a value thats not ready yet but will eventually be a usize , its a promise that i will give you a usize 

 dont run the following instructions until it reolves into its output type

 a Future does nothing until it is awaited 

this code will not print anything because it is not awaited 

```rust 
async fn foo1() -> usize {
    println!("foo")
}

fn main () {
    let _ = foo1()
}
```

Think of futures as exectuing in chunks , while its not ready it lets other things run , it runs untile it cant make progress anymore, then it yields 

Look at this , here we have one threaad for handling terminal reads and one thread for ahndlingc connections, but it gets worse if we have one thread for every operation we have to do 

if one connection is full you ahve to be able to write to the other 


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

- waits on multiple futures and tells you which one finishes first , yiled until something happens 

- uses the idea of cooperative scheduling - if i don;t run i let whoever is above me decide who runs next , nad it might no tbe me 

- cancellation tokens to cancel any future 

- another name proposed for select! was race!

- the way cancellation works is that you describe the circumstances under which you cancel the operation 

- you program can be in an intermediate state it only affects selects, this is an error case you should no 


- fuse means its safe to await this future een though the future has completed in the past 

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

- if you're in an async context and you need to run something that you know might block for a long time , you can use `tokio::task::spawn_blocking` , i'm about to block allow other tasks to run too

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

- only good if you have  afew things, you don;t want a tuple with 100 items lol 

- joinall and tryjoinall uses futures under the hood, join and select allows things to run concurrently, but they don;t allow them to run in parallel

