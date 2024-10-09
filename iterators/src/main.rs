// code from jonwoos livestream about Iterators

fn main() {
    // let mut iter = vec!["a", "b", "c"].into_iter();
    // while let Some(e) = iter.next(){

    // }
    let vs = vec![1, 2, 3];
    // for v in vs {
    //     //consumes vs , owned v
    // }
    // for v in vs.iter() {
    //     //borrows vs , & to v
    // }
    // for v in &vs {
    //     // equivalent as vs.iter()
    // }
}

// You use associated type if you know there is going to be one implementation for the type
// You use generic if you know there are going to be multiple implementations for that given type

// trait Iterator {
//     type Item;
//     fn next(&mut self) -> Option<Self::Item>;
// }

// trait Iterator<Item> {
//     fn next(&mut self)-> Option<Item>;
// }

// trait Service<Request> {
//     fn do(&mut self, r : Request);
// }

// Associated type redues the number of extra generic types you have to implement

// Flatten will iterate over all the inner items in order --> it only recurses one way down

//Lets implement it

// We'll use implied bounds to fix the issue of "where O : Iterator" so that when bounds are on the struct we
// don't need to name it again for every implementation , so that we don't need to propagate the trait bounds up

// we will take the last inner item of the last outer item to implement next_back -> we want to implement the double ended iterator trait

pub fn flatten<I>(iter: I) -> Flatten<I::IntoIter>
where
    I: IntoIterator,
    I::Item: IntoIterator,
{
    Flatten::new(iter.into_iter())
}

//Task --> implement flatmap
//flatmap maps over the outer iterator not the inner iterator
//maps over the outer iterator and the closure that gets given the outer iterator needs to produce the iterator itself
// pub struct Flatten<O, F, I>
// where
//     O:Iterator,
//     F:FnMut(O::Item),
//     I: IntoIterator ,

// we want to implement a trait that allows us to flatten to two levels

// we need to add a trait bound Sized because Flatten needs to know the size of the element it is iterating on at compile time
// Size is the trait that rust used to express that a type has a known size
pub trait IteratorExt: Iterator {
    fn our_flatten(self) -> Flatten<Self>
    where
        //add the size trait
        Self: Sized,
        Self::Item: IntoIterator;
}
impl<T> IteratorExt for T
where
    T: Iterator,
{
    fn our_flatten(self) -> Flatten<Self>
    where
        //add the size trait
        Self: Sized,
        Self::Item: IntoIterator,
    {
        return flatten(self);
    }
}

pub struct Flatten<O>
where
    O: Iterator,
    O::Item: IntoIterator,
{
    outer: O,
    front_iter: Option<<O::Item as IntoIterator>::IntoIter>,
    back_iter: Option<<O::Item as IntoIterator>::IntoIter>,
}

impl<O> Flatten<O>
where
    O: Iterator,
    O::Item: IntoIterator,
{
    fn new(iter: O) -> Self {
        Flatten {
            outer: iter,
            front_iter: None,
            back_iter: None,
        }
    }
}

// looking at the trait bounds , we want the outer thing to be an iterator and we want the items of the outer thing to be intoIterator so we can iterate over them
// when callng next and the outer iterator has been exhausted you need to start walking the back iterator from the front
impl<O> Iterator for Flatten<O>
where
    O: Iterator,
    O::Item: IntoIterator,
{
    type Item = <O::Item as IntoIterator>::Item;
    fn next(&mut self) -> Option<Self::Item> {
        // we'll just do it recursively
        loop {
            //if there is an inner iterator wokr on it, its only if it has been exhausted then we mve on to the next outer iterator
            if let Some(ref mut front_iter) = self.front_iter {
                //if there is a another iterator next
                if let Some(i) = front_iter.next() {
                    return Some(i);
                }
                // or we'll return early with a None
                self.front_iter = None;
            }

            //get the next outer iterator, if thr are no items , then just return
            if let Some(next_inner) = self.outer.next() {
                self.front_iter = Some(next_inner.into_iter())
            } else {
                return self.back_iter.as_mut()?.next();
            }
        }
    }
}

impl<O> DoubleEndedIterator for Flatten<O>
where
    // the outer iterator has to implement DoubleEndedIterator
    O: Iterator + DoubleEndedIterator,

    O::Item: IntoIterator,
    // IntoIter is the iterator type for that item
    <O::Item as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut back_iter) = self.front_iter {
                if let Some(i) = back_iter.next_back() {
                    return Some(i);
                }
                self.back_iter = None;
            }
            if let Some(next_back_inner) = self.outer.next() {
                self.front_iter = Some(next_back_inner.into_iter())
            } else {
                // if we're walkng backwards and the outer iterator yields no more elements then we need to walk the front iterator from the back
                return self.front_iter.as_mut()?.next_back();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(flatten(std::iter::empty::<Vec<()>>()).count(), 0)
    }

    #[test]
    fn empty_wide() {
        assert_eq!(flatten(vec![Vec::<()>::new(), vec![], vec![]]).count(), 0);
    }

    #[test]
    fn one() {
        assert_eq!(flatten(std::iter::once(vec!["a"])).count(), 1)
    }

    #[test]
    fn two() {
        assert_eq!(flatten(std::iter::once(vec!["a", "b"])).count(), 2)
    }

    #[test]
    fn two_wide() {
        assert_eq!(flatten(vec![vec!["a"], vec!["b"]]).count(), 2)
    }
    #[test]
    fn reverse() {
        assert_eq!(
            flatten(std::iter::once(vec!["a", "b"]))
                .rev()
                .collect::<Vec<_>>(),
            vec!["b", "a"]
        )
    }

    #[test]
    fn reverse_wide() {
        assert_eq!(
            flatten(vec![vec!["a"], vec!["b"]])
                .rev()
                .collect::<Vec<_>>(),
            vec!["b", "a"]
        )
    }

    #[test]
    fn both_ends() {
        let mut iter = flatten(vec![vec!["a", "b"], vec!["c", "d"]]);
        assert_eq!(iter.next(), Some("a"));
        assert_eq!(iter.next_back(), Some("d"));
        assert_eq!(iter.next(), Some("b"));
        assert_eq!(iter.next_back(), Some("c"));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None)
    }
    #[test]
    fn inf() {
        let mut iter = flatten((0..).map(|i| 0..i));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
    }

    #[test]
    fn deep() {
        assert_eq!(flatten(flatten(vec![vec![vec![0, 1]]])).count(), 2)
    }
    #[test]
    fn ext() {}
}

// if you decide to flatten eagerly , then you would have to allocate memory , so suppose you hae a scenario where you want to flatten an infinite iterator , then you would fisrt have to allocate an infinite amount of memory
