fn main() {
    //function in rust
    // x in rust is not a fn pointer but a fn item
    // 0 sized value that carries the unique identifer bar at compilet time
    let mut x = bar::<i32>;
    // it returns zero - does not hold a pointer at all
    println!("{}", std::mem::size_of_val(&x));
    //    x = bar::<u32>
    //the fn itm type gets coerced into a fn ptr type
    // fn pointers and fn items are different from one another but fn items are coercable into fn pointers
    //fn item uniquely identifies a function 
    // fn pointer 
    baz(bar::<u32>);
    baz(bar::<i32>);
    // quox(&mut bar::<u32>);
    // non capturing closures are coercable to function pointers 

    // let f = |x:i32, y:i32| x + y;
    let mut z = String::new();
    // let f = || {
    //     println!("{}",z)
    // };
    let f = || {
        z.clear();
    };

    let f = || ();
    // baz(f);
    quox(&f);
}

fn bar<T>(_: u32) -> u32 {
    0
}

//baz takes in a fn pointer
fn baz(f: fn(u32) -> u32) {
    println!("{}", std::mem::size_of_val(&f))
}

fn quox<F>(f:F) 
where F:Fn(), 
{
    (f)()
}

//FnOnce gets a owned reference to self 
// you can't call FnMut from multiple threads at the same time 

//Fnitem or FnPointer does not have state or lifetimes 

fn hello(mut f:Box<dyn FnOnce()>) {
    f()
}