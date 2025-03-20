// macro_rules! vec {
//     ($arg1:ty as $arg2:ident) => {
//         type $arg2 = $arg1;
//     };
//     ($arg1:ty) => {
//         type $arg2 = $arg1;
//     };
// }
// #[macro_export]
// macro_rules! avec {
//     () => {
//         Vec::new()
//     };
//     ($($element:expr),+ $(,)?) => {{
//         let mut vs = Vec::new();
//         $(vs.push($element);)+
//         vs
//     }};
//     // ($element:expr; $count:expr) => {{
//     //     let mut vs = Vec::new();
//     //     let x = $element;
//     //     for _ in 0..$count {
//     //         vs.push(x.clone()); 
//     //     }
//     //     vs
//     // }};
//     ($element:expr; $count:expr) => {{
//         let count = $count;
//         let mut vs = Vec::with_capacity(count);
//         vs.extend(std::iter::repeat($element).take(count));
//         vs
//     }};
// }

#[macro_export]
macro_rules! avec {
    ($($element:expr),*) => {{
        #[allow(unused_mut)]
        let mut vs = Vec::with_capacity($crate::avec![@COUNT; $($element), *]);
        $(vs.push($element);)*
        vs
    }};

    ($(element:expr,)*) => {{
        $crate::avec![$($element), *]
    }};

    (element:expr; $count:expr) => {{
        let mut vs = Vec::new();
        vs.resize($count, $element);
        vs
    }};

    (@COUNT; $($element:expr),*) => {{
        <[()]>::len(&[$($crate::avec![@SUBST; $element]),*])
    }};

    (@SUBST; $_element:expr) => { () };
}
trait MaxValue {
    fn max_value() -> Self;
}

//Just define tha pattern 
macro_rules! max_impl{
    ($t:ty) => {
        impl $crate::MaxValue for $t {
            fn max_value() -> Self {
                <$t>::MAX
            }
        }
    }
}

max_impl!(u32);
max_impl!(i32);
max_impl!(i64);
max_impl!(u64);

fn empty_vec() {
    let x: Vec<u32> = avec![];
    assert!(x.is_empty());
}

#[test]
fn single() {
    let x: Vec<u32> = avec![42, 43];
    assert!(!x.is_empty());
    assert_eq!(x.len(), 2);
    assert_eq!(x[0], 42);
    assert_eq!(x[1], 43);
}

// #[test]
// fn trailing() {
//     let x: Vec<u32> = avec![1,2,3,4,5,6,7,8,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,43];
//     assert!(x.is_empty());
//     assert_eq!(x.len(), 2);
//     assert_eq!(x[0], 42);
//     assert_eq!(x[0], 43);
// }

#[test]
fn trailing() {
    let _: Vec<&'static str> = avec![
        "lakdjwaidjiwalfjhawligfjawilfjawlifwjawliflawilflawlifljawlifaew", "lakdjwaidjiwalfjhawligfjawilfjawlifwjawliflawilflawlifljawlifaew","lakdjwaidjiwalfjhawligfjawilfjawlifwjawliflawilflawlifljawlifaew","lakdjwaidjiwalfjhawligfjawilfjawlifwjawliflawilflawlifljawlifaew",
         "lakdjwaidjiwalfjhawligfjawilfjawlifwjawliflawilflawlifljawlifaew"
    ];
}

#[test]
fn clone_2_nonliteral() {
    let mut y = Some(42);
    let x: Vec<u32> = avec![y.take().unwrap(); 2];

    assert!(!x.is_empty());
    assert_eq!(x.len(), 2);
    assert_eq!(x[0], 42);
    assert_eq!(x[1], 42);
}


//input syntactially valid 
// output valid rust 


//identifiers in the macro world are just completely distinct from the vairables outside of the macro world, that is whre identifiers come in 

