use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::spawn;

const LOCKED: bool = true;
const UNLOCKED: bool = false;

pub struct Mutex<T> {
    locked: AtomicBool,
    v: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> where T : Send + Sync {

}

impl<T> Mutex<T> {
    pub fn new(t: T) -> Self {
        Self {
            locked: AtomicBool::new(UNLOCKED),
            v: UnsafeCell::new(t),
        }
    }
    // fn with_lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
    //     //naive version of the logic 
    //     while self.locked.load(Ordering::Relaxed) != UNLOCKED {}
    //     std::thread::yield_now();
    //     self.locked.store(LOCKED, Ordering::Relaxed);
    //     //Safety: we hold the lock, therefore we can create a mutable refernce 
    //     let ret = f(unsafe { &mut *self.v.get() });
    //     self.locked.store(UNLOCKED, Ordering::Relaxed);
    //     ret 
    // }
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
}


fn main() {
    //we 'll spin up 10 threads each thread is going to increment the value a 100 times 

    //UnsafeCell cannot be shared between threads safely because it does not impl Send or Sync , and Since Mutex is Sync we

    //have to impl Sync for it 
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

    assert_eq!(l.with_lock(|v| *v), 10 * 100);
}


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

fn main() {
    use std::sync::atomic::AtomicUsize;
    let x: &'static _ = Box::leak(Box::new(AtomicBool::new(false)));
    let y: &'static _ = Box::leak(Box::new(AtomicBool::new(false)));
    let z: &'static _ = Box::leak(Box::new(AtomicUsize::new(0)));

    // let _tx = spawn(move || {
    //     x.store(true, Ordering::Release)
    // })
    // let _ty = spawn(move || {
    //     y.store(true, Ordering::Release)
    // })

    // let t1 = spawn(move || {
    //     while !x.load(Ordering::Acquire) {}
    //     if y.load(Ordering::Acquire) {
    //         z.fetch_add(1, Ordering::Relaxed)
    //     }
    // });

    // let t2 = spawn(move || {
    //     while !y.load(Ordering::Acquire) {}
    //     if x.load(Ordering::Acquire) {
    //         z.fetch_add(1, Ordering::Relaxed)
    //     }
    // });
    let _tx = spawn(move || {
        x.store(true, Ordering::SeqCst)
    })
    let _ty = spawn(move || {
        y.store(true, Ordering::SeqCst)
    })

    let t1 = spawn(move || {
        while !x.load(Ordering::SeqCst) {}
        if y.load(Ordering::SeqCst) {
            z.fetch_add(1, Ordering::Relaxed)
        }
    });

    let t2 = spawn(move || {
        while !y.load(Ordering::SeqCst) {}
        if x.load(Ordering::SeqCst) {
            z.fetch_add(1, Ordering::Relaxed)
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();
    let z = z.load(Ordering::SeqCst);

    // What are the possible values for z ? 
    // is 0 possible? if we make is SeqCst , 0 is not possible 
    // Restrictions: 
    //   we know that t1 must run "after" tx
    //   we know that t2 must run "after" ty
    // Given that..
    //   .. tx .. t1 ..
    //     ty t2 tx t1 -> t1 will incr z
    //     ty tx ty t2 t1 -> t1 will incr z
    //   .. tx .. t1 .. t1 ty t2 -> t2 will incr z
    //  Seems impossible to have a thread schedule where z == 0 
    //
    //         t2    t1
    //                v
    // M0(x): false true
    // 
    //         t1
    // M0(y): false true
    // - Is 1 possible?
    // Yes: tx, t1, ty, t2
    // - Is 2 possible?
    // Yes: tx, ty, t1, t2

}