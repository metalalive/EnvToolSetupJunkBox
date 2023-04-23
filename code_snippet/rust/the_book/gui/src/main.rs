use std::any::{Any};
use gui::{Draw, Screen, Button, ProgressBar};

struct SelectBox<'a> {
    w: u32,
    h: u32,
    options: Vec<&'a str>,
}

// TODO: figout out why it has to be static lifetime at here
impl<'a> Draw for SelectBox<'static> {
    fn draw(&self) -> String{
        let firstopt = match self.options.get(0) {
            None => "nothing", Some(s) => s,
        };
        format!("draw a select bbox w={}, h={}, opt-started:{}",
                self.w, self.h, firstopt)
    }
    fn as_any(& mut self) -> & mut dyn Any 
    { self }
}

// Remember the idea behind trait object is type erasure, let different
// low-level detail implementation placed in the concrete types implementing
// particular trait, callers of the trait objects should always focus on the
// high-level concept so downcasting could be prevented as much as possible.
// If you need to downcasting trait objects, consider the alternatives
// like enum type.
fn main() {
    let pgba = ProgressBar::init(108,3);
    let mut out_dev = Screen{
        components:vec![
            Box::new(Button::new(String::from("jet"), 200, 25)),
            Box::new(pgba),
            Box::new(SelectBox{options:Vec::new(), w:201, h:225}),
            Box::new(SelectBox{options:vec!["acid","rock"], w:385, h:529}),
            Box::new(Button::new(String::from("discard"), 390, 75)),
            Box::new(ProgressBar::init(199,10)),
            Box::new(Button::new(String::from("fire"), 313, 34)),
            Box::new(SelectBox{options:vec!["bell","harp"], w:176, h:137}),
        ]
    };
    out_dev.components.push(
        Box::new(Button::new(String::from("\\o_O/"), 70, 16))
    );
    let pgba:& mut Box<dyn Draw> = & mut out_dev.components[1];
    if let Some(pgba) = pgba.as_any().downcast_mut::<ProgressBar>() {
        pgba.percent_done = 0.271;
    } else {
        println!("failed to downcast trait object `Draw` to concrete type `Progressbar`");
    }
    out_dev.run();
    let selector = & mut out_dev.components[3];
    if let Some(selector) = selector.as_any().downcast_mut::<SelectBox>() {
        selector.options.clear();
        selector.options.push("funky");
    }
    out_dev.run();
}
