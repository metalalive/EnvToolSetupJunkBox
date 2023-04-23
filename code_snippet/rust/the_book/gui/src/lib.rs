use std::any::{Any};

// Trait Objects Perform Dynamic Dispatch, the method lookup for each object
// will be performed at runtime
pub trait Draw {
    fn draw(&self) -> String;
    fn as_any(& mut self) -> & mut dyn Any; 
}

pub struct Screen {
    pub components: Vec<Box<dyn Draw>>,
}

impl Screen {
    pub fn run(&self) {
        println!("myscreen: ----- start refreshing ---- ");
        for com in self.components.iter() {
            let databuf:String = com.draw();
            println!("myscreen: {databuf}");
        }
    }
}

pub struct Button {
    pub label:String,
    width:u16,
    height:u16,
}

impl Button {
    pub fn new(txt:String, w:u16, h:u16) -> Self {
        Self {label:txt, width:w, height:h}
    }
}
impl Draw for Button {
    fn draw(&self) -> String {
         format!("draw a button, {}x{}, === {} ===",
                 self.width, self.height, self.label)
    }
    fn as_any(& mut self) -> & mut dyn Any 
    { self }
}

pub struct ProgressBar {
    pub percent_done:f32,
    pub bg_color:u32,
    pub fg_color:u32,
}

impl ProgressBar {
    pub fn init(bgc:u32, fgc:u32) -> Self {
        Self {bg_color:bgc, fg_color:fgc, percent_done:0.0f32}
    }
}
impl Draw for ProgressBar {
    fn draw(&self) -> String {
        format!("draw a progress bar, color:bg={},fg={}, {}"
            , self.bg_color, self.fg_color, self.percent_done)
    }
    fn as_any(& mut self) -> & mut dyn Any 
    { self }
}

