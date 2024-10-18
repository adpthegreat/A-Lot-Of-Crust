fn main() {}
#[derive(Debug)]

// we now have a generic StrSplit implementation that can find itself in a string 
pub struct StrSplit<'haystack, D> {
    remainder: Option<&'haystack str>,
    delimiter: D,
}

// anonymous lifetimes guess that lifetime and theres only one possible lifetime
impl<'haystack, D> StrSplit<'haystack, D> {
    pub fn new(haystack: &'haystack str, delimiter: D) -> Self {
        Self {
            remainder: Some(haystack),
            delimiter,
        }
    }
}

pub trait Delimiter {
    fn find_next(&self, s:&str) -> Option<(usize, usize)>;
}
// using Self is good so that if we rename the types later we don;t hae to change all the types

// if you gave a refernce to something thats on the stack then that reference shouldnt continue living after the function returns
impl<'haystack,D> Iterator for StrSplit<'haystack, D> 
where 
    D:Delimiter,
{
    type Item = &'haystack str;
    fn next(&mut self) -> Option<Self::Item> {
        //ref mut remainder - i want to get a mutable refernce to the thing i'm matching, in this case , remainder
        // &mut remainder - we want to match a reference to the T so its like we have a Option<&mut T>
            //we can also write it like this 
            // let ref mut remainder = self.remainder.as_mut?; or this let reaminder = &mut self.remainder?
            let remainder = self.remainder.as_mut()?;
            if let Some((delim_start, delim_end)) = self.delimiter.find_next(remainder) {
                let until_delimeter = &remainder[..delim_start];
                //we have to dereference it because they are not the same type 
                //&mut &'a &str = &'a &str -- these are the types of the LHS and RHS respecitvely without the referencing of the RHS 
                *remainder = &remainder[delim_end..];
                return Some(until_delimeter);
            } else {
                self.remainder.take()
            }
    }
}

impl Delimiter for &str {
    fn find_next(&self, s:&str) -> Option<(usize, usize)> {
        s.find(self).map(|start| (start, start + self.len()))
    }
}
impl Delimiter for char {
    fn find_next(&self, s:&str) -> Option<(usize, usize)> {
        s.char_indices()
            .find(|(_, c)| c == self)
            .map(|(start, _)| (start, start + self.len_utf8()))
    }
}
//format produces the string, just that the lifetime of the reference we get , is tied to the string 

fn until_char<'s>(s:&'s str, c:char) -> &'s str {
    StrSplit::new(s, c)
    .next()  
    .expect("StrSplit always gives at least one result")
} 

#[test]
fn until_char_test() {
    assert_eq!(until_char("Hello World", 'o'), "Hell");
}
#[test]
fn it_works() {
    let haystack = "a b c d e";
    let letters = StrSplit::new(haystack, " ");
    assert!(letters.eq(vec!["a", "b", "c", "d", "e"].into_iter()));
}
#[test]
fn tail() {
    let haystack = "a b c d e";
    let letters = StrSplit::new(haystack, " ");
    assert!(letters.eq(vec!["a", "b", "c", "d", "e"].into_iter()));
}
// fn empty_tail() {
//     let haystack = "a b c d e";
//     let letters = StrSplit::new(haystack, " ");
//     assert!(letters.eq(vec!["a", "b", "c", "d", "e"].into_iter()));
// }

//items on heap allocation lives until it is dropped - it has lifetime
