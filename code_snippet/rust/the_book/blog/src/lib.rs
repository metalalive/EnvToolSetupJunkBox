
pub struct Post {
    state: Option<Box<dyn State>>,
    content: String,
    capacity_nbytes: usize,
}

impl Post {
    pub fn new (maxnbytes:u16) -> Self {
        let draft = Draft {};
        Self{
            state: Some(Box::new(draft)),
            content: String::new(),
            capacity_nbytes: maxnbytes as usize,
        }
    }
    pub fn add_new_text (&mut self, txt:&str) -> bool {
        let success = (self.content.len() + txt.len()) < self.capacity_nbytes;
        if success {
            self.content.push_str(txt);
        }
        success
    }
    pub fn request_review (&mut self) -> bool {
        if let Some(curr_s) = self.state.take() {
            let next_s = curr_s.req_new_review();
            self.state = next_s;
            self.state.is_some()
        } else {
            false
        } 
    }
    pub fn approve (&mut self) -> bool {
        if let Some(curr_s) = self.state.take() {
            self.state = curr_s.do_approve();
            self.state.is_some()
        } else {
            false
        } 
    }
    pub fn content (&self) -> & str {
        match &self.state {
            Some(s) => s.detailtxt(self),
            None => "invalid state",
        }
    }
} // end of impl Post

// - the first argument of trait functions could also be Box smart
//   pointer with any type in addition to `self` or `&self`.
// - That means this function can be invoked only when the trait
//   object is wrapped in another Box pointer.
// - Some functions in the `State` trait are for handling state
//   transitions, the idea here is to let each transition function
//   takes the ownership of Box pointer wrapping the previous state
//   (a concrete type), then return the Box pointer wrapping the
//   next state (another concrete type)
// - in other words, callers can invoke each transition function
//   only once due to the ownership movement of the state object,
//   then the old state is immediately invalidated after the call,
//   and caller is not allowed to use the old state again.
trait State {
    fn req_new_review(self:Box<Self>) -> Option<Box<dyn State>>;
    fn do_approve(self:Box<Self>) -> Option<Box<dyn State>>;
    fn detailtxt<'a> (&self, _p:&'a Post) -> &'a str
    { "" }
}

struct Draft {}
struct PendingReview {}
struct Published {}

impl State for Draft {
    // simply abandon old state object
    fn req_new_review(self:Box<Self>) -> Option<Box<dyn State>>
    { Some(Box::new(PendingReview{})) }
    fn do_approve(self:Box<Self>) -> Option<Box<dyn State>>
    { Some(self) }
}

impl State for PendingReview {
    // no need to create another pending review, return the same state
    fn req_new_review(self:Box<Self>) -> Option<Box<dyn State>>
    { Some(self) }
    fn do_approve(self:Box<Self>) -> Option<Box<dyn State>>
    { Some(Box::new(Published{})) }
}

impl State for Published {
    // published post should not have pending reviews, return the same state
    fn req_new_review(self:Box<Self>) -> Option<Box<dyn State>>
    { Some(self) }
    fn do_approve(self:Box<Self>) -> Option<Box<dyn State>>
    { Some(self) }
    fn detailtxt<'a> (&self, p:&'a Post) -> &'a str
    { p.content.as_str() }
}

