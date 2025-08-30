struct Rc<T> {
    inner: *mut Inner<T>
}

//smol test 
// fn foo<T: Send>(_: T) {}

// fn bar(x: Rc<()>) {
//     foo(x)
// }

// this works
struct MutexGuard<'a, T> {
    i: &'a mut T,
    _not_send: std::marker::PhantomData<Rc<std::rc::Rc<()>>>
}

// this does not work 
// impl<T> !Send for MutexGuard<'_,T> {}

struct Inner<T> {
    count: usize,
    value: T,
}

impl <T> Rc<T> {
    pub fn new(v : T) -> Self {
        Rc {
            inner : Box::into_raw(Box::new(Inner {
                count: 1,
                value : v,
            }))
        }
    }
}

impl<T> Clone for Rc<T> {
    fn clone(&self) -> Self {
         unsafe { &mut *self.inner}.count += 1; // this is safe
         Rc {
            inner: self.inner
         }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        let cnt = &mut unsafe { &mut *self.inner }.count;
        if *cnt == 1 {
            let _ = unsafe { Box::from_raw(self.inner)};
        } else {
            *cnt -= 1;
        }
    }
}

impl<T> std::ops::Deref for Rc<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        return &unsafe { &*self.inner }.value; //self.inner is a raw pointer 
    }
}


fn main() {
    let x = Rc::new(1);
    let y = x.clone();

    // std::thread::spawn(move || {
    //     drop(y); //anything the thing in the closure has to be Send 
    // });
    // drop(x);
}

// Race condition
// they both end up seeing 1 and dropping it leading to undefined behaviour 