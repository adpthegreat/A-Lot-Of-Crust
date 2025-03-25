use std::future::Future;
use async_trait::async_trait;
use tokio::sync::Mutex as TMutex;
struct Request;
struct Response;

#[async_trait]
trait Service {
    async fn call(&mut self,_:Request) -> Response;
}

struct X;

#[async_trait]
impl Service for X {
    async fn call(&mut self, _: Request) -> Response {
        Response
    }
}

async fn main() {
    // let runtime = tokio::runtime::Runtime::new();
    // runtime.block_on(async{
    //     println!("Hello, world");

    //     let mut accept = tokio::net::TcpListener::bind("0.0.0.0:8080");
    //     while let Ok(stream) = accept.wait {
    //         tokio::spawn(handle_connection(stream));
    //     }
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





