

## Notes on macros from jon gjengset 


## Reducing repitive code
imagine a scenario we want differnt impl for different number typees of a particular trait that has a max_value method, instead of doing all this -> which starts to get repetitive , we can just create a macro for it by defining the pattern and then use it 

its better because what if we wanted to change the implemetnation then we have to change the implementation for every type manually

So instead of doing this 

```rust 

trait MaxValue {
    fn max_value() -> Self;
}

impl MaxValue for u32 {
    fn max_value() -Self {
        u32::Max
    }
}
impl MaxValue for i32 {
    fn max_value() -Self {
        i32::Max
    }
}
impl MaxValue for u64 {
    fn max_value() -Self {
        u64::Max
    }
}
```

We can just create a macro for it and just do this 
```rust 
trait MaxValue {
    fn max_value() -> Self;
}


//Just define tha pattern 
macro_rules! max_impl{
    ($t:ty) => {
        impl $crate::MaxValue for $t {
            fn max_value() -> Self {
                <$t>::Max
            }
        }
    }
}

max_impl!(u32);
max_impl!(i32);
max_impl!(i64);
max_impl!(u64);

```

Now this is the generated rust code when we use `cargo expand --lib` 

```rust 

#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
trait MaxValue {
    fn max_value() -> Self;
}
impl crate::MaxValue for u32 {
    fn max_value() -> Self {
        <u32>::Max
    }
}
impl crate::MaxValue for i32 {
    fn max_value() -> Self {
        <i32>::Max
    }
}
impl crate::MaxValue for i64 {
    fn max_value() -> Self {
        <i64>::Max
    }
}
impl crate::MaxValue for u64 {
    fn max_value() -> Self {
        <u64>::Max
    }
}
fn empty_vec() {
    let x: Vec<u32> = Vec::new();
    if !x.is_empty() {
        ::core::panicking::panic("assertion failed: x.is_empty()")
    }
}
```



## Macro expansion does subtitution 

Suppose we have a macro rule such as this 

```rust 
#[macro_export]
macro_rules! avec {
    () => {
        Vec::new()
    };
    ($($element:expr),+ $(,)?) => {{
        let mut vs = Vec::new();
        $(vs.push($element);)+
        vs
    }};
    ($element:expr; $count:expr) => {{
        let mut vs = Vec::new();
        for _ in 0..$count {
            vs.push($element);
        }
        vs
    }};
}

```

if we try to run tests on it for cases like this it suceeds, the vec is created , the value are in the vec , len is asserted , etc 

```rust 
#[test]
fn single() {
    let x: Vec<u32> = avec![42, 43];
    assert!(!x.is_empty());
    assert_eq!(x.len(), 2);
    assert_eq!(x[0], 42);
    assert_eq!(x[1], 43);
}

```

if we write tests like this, it fails 

```rust
#[test]
fn clone_2_nonliteral() {
    let mut y = Some(42);
    let x: Vec<u32> = avec![y.take().unwrap(); 2];

    assert!(!x.is_empty());
    assert_eq!(x.len(), 2);
    assert_eq!(x[0], 42);
    assert_eq!(x[1], 43);
}


//output 

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/lib.rs (target/debug/deps/macros-b5a6bf2f435a7e70)

running 3 tests
test single ... ok
test trailing ... ok
test clone_2_nonliteral ... FAILED

failures:

---- clone_2_nonliteral stdout ----
thread 'clone_2_nonliteral' panicked at src/lib.rs:89:38:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    clone_2_nonliteral

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

but why? Now what happens is that the for loop generated from the third macro rule for this expr will call .take().unwrap() on each of the elements, so it panics when we reach the second one because the value is None (there are no more elements), we can see the generated code  with `cargo expand --lib --tests`


```rust 
fn clone_2_nonliteral() {
    let mut y = Some(42);
    let x: Vec<u32> = {
        let mut vs = Vec::new();
        for _ in 0..2 {
            vs.push(y.take().unwrap()); // only the first take will suceed
        }
        vs
    };
    if !!x.is_empty() {
        ::core::panicking::panic("assertion failed: !x.is_empty()")
    }
    ///rest of the code 
}
```

to fix this we can creae modify the macro_rule such that the expression tht we pas is only evaluated a single time and we clone it each time we push 


```rust 
    ($element:expr; $count:expr) => {{
        let mut vs = Vec::new();
        let x = $element;
        for _ in 0..$count {
            vs.push(x.clone()); // be aware that macro exapnsion does substitution 
        }
        vs
    }};
```

## Macro tricks 


This is the generated code 

```rust 
fn single() {
    let x: Vec<u32> = {
        #[allow(unused_mut)]
        let mut vs = Vec::with_capacity({ <[()]>::len(&[(), ()]) });
        vs.push(42);
        vs.push(43);
        vs
    };
    if !!x.is_empty() {
        ::core::panicking::panic("assertion failed: !x.is_empty()")
    }
    //Rest of the code 
}
```


https://doc.rust-lang.org/src/alloc/macros.rs.html#42




## Little book of rust macros notes

//Macro processing happens after AST construction

//First AST construction macro expansion

//The compiler treats the exapnsion of a macro as an AST node and just a mere sequence of tokens

//macro expansion happens in phases and has a limit called a 'macro recursion limit' - 32

//macro_rules! $name {
    $rule0 ;
    $rule1 ;
    $ruleN ;
}

($pattern) => {$expansion}

## Matching 

//when a amcro invoked , rules are gone through one by one, contents of the input token tree are matched against the rules pattern 

// if input matches pattern then the invocation is replaced with the output

## Captures 
Patterns can also contain captures. allows nputs to be matched based on some general grammar category 

Captures are written as a dollar ($) followed by an identifier, a colon and the type of capture , which must be one of the following 

`item` - an item 
`block` - a block (block of statements and an expression, surrounded by braces)
`stmt` - a statement
`pat` - a pattern 
`expr` - an expression 
`ty` - a type
`ident` - an identifier
`path` - a path eg  std::mem::replace
`meta` - a meta item; things that go inside #[...] and #![...] attributes
`tt`  single token tree 

`$name:kind` - capture syntax

```rust 
 macro_rules! one_expression {
    ($e:expr) => {...};
 }
```

captures are substituted as complete AST nodes. This means no matter what sequence of tokens is captured by $e , it is interpreted as a single, complete expression.

## Repititions 
Patterns can contain repitions this allows a sequence of tokens to be matched , they have the general form `$(...) sep rep`

`$` - literal dollar token 
`...` - pattern being repeated 
`sep` - separator
`rep` repeat control , its either * or + , which means zero or more (kleene star stuff here lol) or one or more repeats 


## Captures and expansionn redux 
Once the parser begins consuming tokens for a capture, it cannot stop or backtrack

### token trees vs AST nodes
```rust 
macro_rules! capture_expr_then_stringify {
    ($e:expr) => {
        stringify!($e)
    };
}


fn main() {
    println!("{:?}", stringify!(dummy(2 * (1 + (3)))));
    println!("{:?}", capture_expr_then_stringify!(dummy(2 * (1 + (3)))));
}
```

`stringify` is a built in syntax_extension that takes all the tokens it is given and concatenates it into one big string 

despite having the same input, the output is different because upon invocation, the first one stringifies a sequence of token trees, while the second one is stringifying an AST expression node 

## Hygiene

- each macro is given a new unique syhntax context for its contents