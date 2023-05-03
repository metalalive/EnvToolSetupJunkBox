use std::collections::HashMap;
use std::fs::File as LocalFile;
use std::io::{ErrorKind, Error as IOError};
use std::cmp::PartialOrd; // a trait which implements compare operator e.g. >, <, >=
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{Deref, DerefMut, Drop, Add};
use std::rc::{Rc, Weak};
use std::cell::{RefCell, RefMut, Ref, Cell};
use std::slice;
// use std::marker::Sized;

fn owner_ref_demo() {
    println!("-------- reference demo --------");
    let mut x:u8 = 13;
    let y = & mut x; // mutable reference assgined to `y`
    // cannot borrow immutable reference here, cuz already borrow mutable reference
    // assert_eq!(13, x);
    assert_eq!(13, *y);
    *y += 5;
    assert_eq!(18, *y);
    // the immutable reference `y` is no longer used, it is safe to use `x` again 
    assert_eq!(18, x);
    println!("-------- ownership demo --------");
    let s1 = String::from("fire-cracker");
    let s2 = s1 .clone();
    println!("s1:{}, s2:{}, init len:{}", s1, s2, s1.len()); // length is 12
    let mut s3 = s2;
    println!("s1:{}, s3:{}", s1, s3);
    { // note String tpye doesn't support indexing due to the encodings
      // in different human languages
        let s3p1 = & mut s3;
        s3p1.replace_range(5..9, "gooph");
        println!("s3p1:{}, len:{}", s3p1, s3p1.len()); // length is 13 ?
    } // TODO, why increasing string length ?
    let s3p2 = & mut s3;
    s3p2.replace_range(0..4, "soda");
    println!("s3p2:{}", s3p2);
    let s3p3 = & s3;
    println!("s3p3:{}", s3p3);
    println!("s1:{}", s1);
} // end of owner_ref_demo


#[derive(Debug)]
struct Vertex3D(f32,f32,f32);

#[derive(Debug)]
struct Rectangle {
    width :u16,
    height:u16,
}

impl Rectangle {
    fn generate(w:u16, h:u16) -> Self {
        Self {width:w,height:h}
    }
    fn area(&self) -> u32 {
        let a:u32 = u32::from(self.width) * u32::from(self.height);
        a
    } // u32::from() cannot be used in generic type
    fn scale(& mut self, pp:(i32,i32)) {
        let w = i32::from(self.width)  + pp.0;
        let h = i32::from(self.height) + pp.1;
        self.width  = w as u16; // (cast i32 to u16, omit upper 16-bit)
        self.height = h as u16;
        // syntax `as` is only for primitive type conversion, not generic type
    } // argument `self` is mutable reference, its attributes can be directly modified
} // end of associated functions of the struct

fn struct_demo() {
    println!("-------- struct usage --------");
    let mut r0 = Rectangle {
        width: 1920,
        height: 1080,
    };
    let r1 = Rectangle::generate(48,50);
    println!("r0 content: {:?}, area:{}", r0, r0.area());
    println!("r1 content: {:?}, area:{}", r1, r1.area());
    r0.scale((-23,-45));
    // Rust will be automatically de-referencing, by adding syntax `*` to the struct instance
    println!("r0 content: {:?}, area:{}", r0, (&r0).area());
    let mut v3d = Vertex3D(12.3, -4.56, 789.0);
    assert!(12.35 > v3d.0 && 12.25 < v3d.0);
    assert!(789.05 > v3d.2 && 788.95 < v3d.2);
    v3d.2 = 9.75;
    assert!(9.755 > v3d.2 && 9.745 < v3d.2);
}

// each variant of a enum type can carry different data / struct
// type, struct is implicitly declared at here
#[derive(Debug)]
enum MqttCmdMsg{
    Disconnect,
    Puback,
    Publish {content:String, qos:u8},
    Consume(Option<String>), // the value can be either string or null
    Connect(u8, Option<i8>, u16, bool),
}

fn mq_cmd_scalar(cmd_in:MqttCmdMsg) -> i8 {
    match cmd_in { // note the input variable is moved here
        MqttCmdMsg::Publish {content:mcontent, qos:mqos} => {
            let slen = mcontent.len() as i8;
            let q = mqos as i8;
            slen + q
        },
        MqttCmdMsg::Consume(maybe_s) =>
            match maybe_s {
                None => -2 as i8,
                Some(s0) => s0.len() as i8,
            },
        MqttCmdMsg::Connect(dd, _, yy, _) => {
            let d2 = dd as i8; // extract lower 7 bits (note it is 8-bit signed integer)
            let y2 = yy as i8;
            println!("mq_cmd_scalar, dd={}, d2={}, yy={}, y2={}", dd, d2, yy, y2);
            d2.saturating_add(y2) // add until the numeric upper bound is reached
        },
        MqttCmdMsg::Disconnect => 0x5e as i8,
        _other => -1i8,
    }
}

fn enum_demo() {
    println!("-------- enum usage --------");
    let disconn = MqttCmdMsg::Disconnect;
    let m_pub_ack = MqttCmdMsg::Puback;
    let m_publish = MqttCmdMsg::Publish {content:String::from("do-re-mi"), qos:12} ;
    // the values of enum variant seem always immutable once specified ?
    let mut m_conn = MqttCmdMsg::Connect(18, Some(-49), 500, true);
    let m_consumer_userdef_prop = String::from("code flying drone");
    let mut m_consumers = [
        MqttCmdMsg::Consume(None),
        MqttCmdMsg::Consume(Some(m_consumer_userdef_prop)), // string moved at here
    ];
    println!("m_conn: {:?}, disconn: {:?}, m_publish:{:?}, m_pub_ack:{:?}",
             m_conn, disconn, m_publish, m_pub_ack);
    for item in m_consumers.iter(). enumerate() {
        let (idx, consumer): (usize, &MqttCmdMsg) = item;
        println!("m_consumers[{}]: {:?}", idx, consumer);
    } // iterate by reference, to avoid implicit moves
    m_consumers.swap(0,1);
    m_consumers[1] = MqttCmdMsg::Consume(Some(String::from("flight tickets")));
    // m_consumers[1].push_str(" and boost"); // this won't work
    m_conn = MqttCmdMsg::Connect(25, None, 7801, false);
    println!("m_conn: {:?}",  m_conn);
    println!("m_consumers[0]: {:?}", m_consumers[0]);
    println!("m_consumers[1]: {:?}", m_consumers[1]);
    println!("scalar, m_publish: {}", mq_cmd_scalar(m_publish));
    println!("scalar, m_conn: {}", mq_cmd_scalar(m_conn));
    println!("scalar, m_pub_ack: {}", mq_cmd_scalar(m_pub_ack));
    let [_, consumer1] = m_consumers; // destruct
    println!("scalar, consumer1: {}", mq_cmd_scalar(consumer1));
} // end of enum_demo

fn vector_demo() {
    println!("-------- collection vector usage --------");
    let mut v:Vec<u8> = Vec::new();
    v.push(91);
    v.push(188);
    println!("v size: {}, capacity:{}", v.len(), v.capacity());
    v.push(175);
    v.push(204);
    v.push(206);
    println!("v size: {}, capacity:{}", v.len(), v.capacity());
    // use get() when caller might occasionally assess items out of range
    println!("v[0...3] content: {:?}", v.get(0..3));
    println!("v[2...4] content: {:?}", v.get(2..4));
    println!("v[4...8] content: {:?}", v.get(4..8));
    println!("v[999] content: {:?}", v.get(999));
    for i in & mut v { // mutable reference interator
        *i += 1;
    }
    let last_item:Option<&mut u8> = v.get_mut(4);
    match last_item {
        None => println!("last-item points to nothing"),
        Some(value_ref) => {*value_ref -=  10;}
    };
    // remember that any value cannot have mutable borrows and
    // immutable borrows interleaving in a code section
    let last_item_borrow = &v[4];
    println!("last_item_borrow: {:?}", last_item_borrow);
    assert_eq!(v, [92,189,176,205,197]);
    println!("v content: {:?}", v);
    v.pop();
    println!("v size: {}", v.len());
    println!("v content: {:?}", v);
} // end of vector_demo

fn string_demo() {
    println!("-------- collection string usage --------");
    let s1 = "Backtoback".to_string(); // convert string literal to instance of a String type
    let s2 = "Handinhand"; // string literal
    // copy raw bytes referenced by `&s2`, to internal buffer of `s1`
    // string concatenation like this may reduce number of unnecessary memory re-allocation
    let mut s3 = s1 + &s2;
    // s1 is invalid from here, s3 took the ownership
    println!("s2: {}, s3: {}", s2, s3);
    s3.push(' ');
    s3.push_str("HereAndThere");
    // slice structure actually stores only starting position and
    // length of the slice. It is also considered as one of usages
    // in Dynamically Sized Type
    println!("s3: {}, slice[4..10]: {}, slice[25..]: {}",
             s3, &s3[4..10], &s3[25..]);
} // string indexing is prohibited, but range selection is allowed.

fn hashmap_demo() {
    println!("-------- hash map usage --------");
    let green_str = String::from("green"); // String type
    // string literal does't have ownership ?
    let blu_str = "blue";
    let rnbw_str = "rainbow";
    let blu_t_score = 276;
    let mut scores = HashMap::new();
    // 1. if data type of key is not specified initially, the first insertion will determine that
    // 2. keys could be integer, string literal, String type
    // 3. if a key is String type, the hash map will take ownership of the key string
    scores.insert(green_str, 0x87); // `scores` takes ownership of `green_str`
    scores.insert(blu_str.to_string(), blu_t_score);
    // scores.insert(blu_str, blu_t_score);
    let actualvalue = scores.get(rnbw_str) .copied() .unwrap_or(0);
    assert_eq!(actualvalue, 0);
    assert_eq!(scores.contains_key(rnbw_str), false);
    let actualvalue = scores.get(blu_str) .copied() .unwrap_or(0);
    assert_eq!(actualvalue, blu_t_score);
    // overwrite the hash netry
    let blu_t_score = 329;
    scores.insert(blu_str.to_string(), blu_t_score);
    let actualvalue = scores.get(blu_str) .copied() .unwrap_or(0);
    assert_eq!(actualvalue, blu_t_score);
    // check then add
    scores.entry("purple".to_string()) .or_insert(539);
    println!("scores in all teams: {:?}", scores);
    scores.remove(blu_str);
    assert_eq!(scores.contains_key(blu_str), false);
} // end of hashmap_demo

fn _file_process_propagate_err(filepath:&str) -> Result<u32, IOError>
{// the ? operator would return error or the integer below
    let _f = LocalFile::open(filepath)?;
    Ok(0x1234 as u32)
} // automatically close the file out of the scope

fn _last_char_first_line(text:&str) -> Option<char>
{ // the ? operator would return None if no char is read
    text.lines().next()? .chars() .last()
}

fn recoverable_error_demo() {
    println!("-------- recoverable error demo --------");
    let filepath = "/path/to/non/exist/file";
    let myresult = LocalFile::open(filepath);
    match myresult {
        Ok(f) => println!("file open succeeded:{:?}", f),
        Err(e) => match e.kind() { // error in myresult is partially moved at here
            ErrorKind::NotFound => println!("failed to find the file"),
            ErrorKind::PermissionDenied => println!("no access to the file"),
            other_error => println!("unclassified error on file open, reason:{:?}",
                             other_error)
        },
    };
    // myresult.expect(), and unwrap() will cause panic if error is returned
    // expect(), unwrap() can be considered as types of assert() in other languages
    // let f2 = LocalFile::open(filepath).expect("report file error in result.expect()");
    // println!("f2: {:?}", f2);
    let myresult = _file_process_propagate_err(filepath);
    println!("myresult: {:?}", myresult);
    assert_eq!(_last_char_first_line(""), None);
    assert_eq!(_last_char_first_line("\nxJhyu"), None);
    assert_eq!(_last_char_first_line("yua\nxJhyu"), Some('a'));
    assert_eq!(_last_char_first_line("chet\nK993"), Some('t'));
} // end of recoverable_error_demo

// can also be multiple generic types, each has different name
fn _find_greatest_num<T:PartialOrd>(list:&[T]) -> &T
{
    let mut out:&T = &list[0];
    for n_ptr in list {
        if n_ptr > out { out = n_ptr; }
    }
    out
}

fn generic_func_demo() {
    println!("-------- generic function demo --------");
    let list_caller:[i8;5] = [-100,15,38,-82,40,];
    assert_eq!(*_find_greatest_num(&list_caller), 40);
    let list_caller:[i16;8] = [-1105,-1114,-1102,39,150,-256,169,-7];
    assert_eq!(*_find_greatest_num(&list_caller), 169);
    let list_caller:[char;7] = ['A',' ','E','a','Z','m','k'];
    assert_eq!(*_find_greatest_num(&list_caller), 'm');
}


// required method in Display trait can be omitted at here
// , but concrete types must implement all the required methods
// in isolated block, e.g. `impl Display for xxx`
trait ArticleSummary :Display {
    fn summarize(&self) -> String;
}

struct DailyNews {
    headline:String,
    location:String,
    author:String,
    content:String,
}
impl ArticleSummary for DailyNews {
    fn summarize(&self) -> String {
        format!("{}, by {} - {}", self.headline,
                self.author, self.location)
    }
}
impl Display for DailyNews {
    fn fmt(&self, f:&mut Formatter<'_>) -> FmtResult
    { write!(f, "{}",  self.summarize()) }
}

struct Tweet {
    username:String,
    content:String,
    retweet:bool,
    num_replies:u16,
    summary_limit:u16,
}
impl ArticleSummary for Tweet {
    fn summarize(&self) -> String {
        let fullcontent = self.content.as_str();
        let mut sz:usize = fullcontent.len();
        let maxlimit:usize = self.summary_limit as usize;
        sz = if sz > maxlimit {maxlimit} else {sz};
        let content_begin = &fullcontent[0..sz];
        // macro argument must not contain variables relied on runtime
        format!("{}({})({}): {} ", self.username, self.retweet,
            self.num_replies, content_begin )
    } // rangle selection is not allowed at compile time
}
impl Display for Tweet {
    fn fmt(&self, f:&mut Formatter<'_>) -> FmtResult
    { write!(f, "{}",  self.summarize()) }
}


// return a pointer to instance of any type which implements the ArticleSummary trait
fn caller_generate_article(switch:bool) -> Box<dyn ArticleSummary>
{ // this function cares only the return types implementing the trait `ArticleSummary`
  // , all structs implementing `ArticleSummary` may have various sizes, compiler
  // will report error due to the size difference. So the instances of the structs
  // should be wrapped in the instance of smart pointer Box<T>
    if switch {
        Box::new(Tweet {
            username:String::from("nagazawa"),
            content:"a new milestone of xyz project, age of wooden horse".to_string(),
            retweet:false,  summary_limit:22,  num_replies:641,
        })
    } else {
        Box::new(DailyNews {
            headline:"global inflation".to_string(),
            location:"Yunlin, Taiwan".to_string(),
            author:"Olivia Janbossiem".to_string(),
            content:"abc abc abc abc".to_string(),
        })
    }
} // end of  caller_generate_article

// - the function uses trait bound on generic type, compiler will generate
//   non-generic implementation for each concrete type, this is called `static dispatch`
// - `?Sized` is used for relaxing the restriction and ignoring the bound to `Sized`
//    trait, allow the size of given arguement `item`  to be unknown.
fn notify_article<T:ArticleSummary + Display + ? Sized> (item:&T)
{ // the function cannot know size of the input type implementing the traits
  // at compile time, however that should be ok in this case cuz the input
  // is a reference
    println!("notifying article : {}", item.to_string());
}

fn trait_demo() {
    println!("-------- trait demo --------");
    // after generating a new instance, the function returns points to dynamic types
    //  which implement the same trait.
    let onetweeet = caller_generate_article(true);
    //let onetweeet = & onetweeet;
    let report    = caller_generate_article(false);
    // Typically the syntax `*` is a dereference from instance of Box smart pointer
    // to instance of original type.
    // However, the type size is unknown at compile time, that will cause compile error.
    // The alternative is to immediately convert them to references.
    notify_article(& * onetweeet);
    notify_article(& * report);
    // Note the golden rule of dynamically sized type is to put value of the
    // unknown-sized type behind the pointer `&`
}

// Rust's borrow checker at compile time cannot determine :
// 1. whether reference of either s1 or s2 will be returned
// 2. the concrete lifetime of returned reference, whether it is still
//    valid after the function call
// 3. how the references of s1 or s2  relate to the returned reference
fn find_longest_str<'a>(s1:& 'a str, s2:& 'a str) -> & 'a str {
    if s1.len() > s2.len() {s1} else {s2}
}

fn  lifetime_variable_demo()
{
    println!("-------- lifetime demo --------");
    let mylist:[&str;4] = [ "warmup", "standup", "takedown", "out-of-order", ];
    let result;
    assert_eq!(find_longest_str(mylist[0], mylist[2]), mylist[2]);
    assert_eq!(find_longest_str(mylist[2], mylist[3]), mylist[3]);
    { // inner scope, `String` instance will cause compile error, string literal won't
      // , string literals have static lifetime, they're stored directly in program
      // binary (e.g. data section)
        let short_lived_str = "black-coffee";
        result = find_longest_str(short_lived_str , mylist[2]);
        // let short_lived_str = "black-coffee".to_string();
        // result = find_longest_str(short_lived_str.as_str(), mylist[2]);
        assert_eq!(result, short_lived_str);
    }
    println!("result: {}", result);
}


fn functional_closure_demo_case_1()
{ // define closure as normal function, without capturing any value in the scope
    let mut mylist:Vec<i16> = vec![190,3874,-203];
    let simple_func = |x:& mut Vec<i16>, num:i16| -> usize {
        x.push(num);
        x.len()
    };
    assert_eq!(simple_func(& mut mylist, 1349), 4);
    assert_eq!(simple_func(& mut mylist, -918), 5);
    let mylist2:Vec<i16> = mylist;
    let simple_func = |mut x:Vec<i16>| -> Vec<i16> {
        let elm = x.remove(0); // potential panic might happen
        x.push(1);
        x.push(elm);
        x
    };
    let mylist3 = simple_func(mylist2);
    assert_eq!(mylist3.len(), 6);
    let mylist4 = simple_func(mylist3);
    assert_eq!(mylist4.len(), 7);
    println!("mylist4 : {:?}",mylist4);
}
fn functional_closure_demo_case_2()
{ // try mutable borrow
    let mut list = vec![1, 2, 3];
    println!("Before defining closure: {:?}", list);
    // unlike case 1, this case captures the `list` without defining
    // input argument in the clusure
    let mut borrows_mutably = |n| list.push(n);
    // between the declaration and the final call to the same closure, there must not
    // be any immutable borrow on the `list`, otherwise it will get compile error
    borrows_mutably(7); // the first call determines type of input arguments
    borrows_mutably(54); // subsequent calls have to use the same type in their inputs
    borrows_mutably(124);
    println!("After calling closure, immutable borrow on list: {:?}", list);
}
fn functional_closure_demo_case_3()
{ // try FnOnce trait implemented in Option<T>::unwrap_or_else()
    let mut maybenull:Option<i32> = None;
    let default_err_val:i32 = -2;
    let give_default_val = || -> i32 {
        println!("FnOnce is called, {:?}", maybenull);
        default_err_val // return
    }; // automatically capture and borrow `maybenull`, all occur in this closure
    assert_eq!(maybenull.unwrap_or_else(give_default_val), default_err_val);
    assert_eq!(maybenull.unwrap_or_else(give_default_val), default_err_val);
    maybenull = Some(-92018);
    // cannot use the sane closure `give_default_val` again later after another value
    // is assigned to variable `maybenull`, it will cause compile error
    //// assert_eq!(maybenull.unwrap_or_else(give_default_val), -92018);
    assert_eq!(maybenull.unwrap_or_else(|| default_err_val as i32), -92018);
}
fn fnonce_implementor<F>(func:F) -> i8 where F:FnOnce() -> String
{
    println!("fnonce_implementor is calling func ptr: {}", func());
    // after `func` is called, it will be moved (where ?) and `func` is invalid
    // cannot be called again, otherwise compiler will report error
    0 
}
fn functional_closure_demo_case_4()
{ // try FnOnce trait with given closure which moves ownership of a value in outer scope
    let mut x:String = "will be moved" .to_string();
    let f = move || {
        x.push_str(" once closure is called");
        x
    };
    assert_eq!(fnonce_implementor(f) , 0); // move ownership of `x` to the function
    // `x` is invalid and cannot be used again, otherwose compiler will report error
    // assert_eq!(fnonce_implementor(f) , 0);
}
fn functional_closure_demo_case_5()
{ // try FnMut trait implemented in slice::sort_by_key() 
    let mut rectangles:[Rectangle;4] = [
        Rectangle {width:216, height:89},        Rectangle {width:90,  height:818},
        Rectangle {width:173, height:404},       Rectangle {width:185, height:416},
    ];
    let mut num_compares = 0;
    let f1 = |r:&Rectangle| -> u16 {num_compares += 1;  r.height};
    // wlll be invoked several times
    rectangles.sort_by_key(f1);
    println!("the sorted rectangles by their height: {:?}, #compares:{}",
             rectangles, num_compares);
}

fn functional_closure_demo()
{
    println!("-------- functional closure demo --------");
    functional_closure_demo_case_1();
    functional_closure_demo_case_2();
    functional_closure_demo_case_3();
    functional_closure_demo_case_4();
    functional_closure_demo_case_5();
} // end of functional_closure_demo


fn functional_iterator_demo()
{
    println!("-------- functional iterator demo --------");
    let mut l = vec![91, 23, 567];
    let mut l2 = l.iter();
    // the thing we get is an immutable reference to the value
    assert_eq!(l2.next() , Some(&91));
    assert_eq!(l2.next() , Some(&23));
    assert_eq!(l2.next() , Some(&567));
    assert_eq!(l2.next() , None); // to repeat again (endlessly) , call Iter::cycle()
    // iterate all entries and apply the closure in map at once
    let l2:Vec<_> = l.iter().map(|x| x - 1).collect();
    assert_eq!(l2, vec![90, 22, 566]);
    // take closure and invoke it with each value in the vec
    l.remove(0);
    let mut l2 = l.iter().map(|x| x - 10);
    assert_eq!(l2.next() , Some(13));
    assert_eq!(l2.next() , Some(557));
    assert_eq!(l2.next() , None);
    // filter out all negative integers
    let l:Vec<i16> = vec![91, -23, 56, -7, 890, -12, 345, -67];
    // iterate all entries and apply the closure in filter at once
    let l2:Vec<&i16> = l.iter().filter(
        |x:&&i16| -> bool {
            let z:i16 = 0 as i16;
            let x2:i16 = **x;
            z < x2
        } // TODO, why using two borrows here
    ).collect();
    assert_eq!(l2, vec![&91, &56, &890, &345]);
} // end of  functional_iterator_demo


// Box smart pointer is applied when the size of type is
// unknown at compile time.
// If the value is not reference, Box smart pointer MUST be
// the ONLY owner of the value.
// so Box might not be good option for vertices in graph dat
// a structure ? since there might be several inbound / outbound
// links for a vertex
#[derive(Debug)]
enum ExListNode {
    Cons(u32, Box<ExListNode>),
    Nil
}
fn smart_ptr_box_demo()
{
    use ExListNode::{Cons, Nil};
    println!("-------- smart-pointer box demo --------");
    let final_node = Cons(4, Box::new(Nil));
    let head_node:ExListNode = Cons(98, Box::new(Cons(765, Box::new(final_node))));
    let cl_p = &head_node; // cl_p references to the value in `head_node`
    println!("cl_p: {:?}", *cl_p);
    // you can also specify reference in match statement, so Rust compare
    // the value referenced by `cl_p` with moveing the variable `head_node`
    match  cl_p {
       ExListNode::Cons(val, nxt_p) => {
           println!("--- val:{}, next pointer:{:?}", val, nxt_p);
       },
        _other => {println!("end of cons list");}
    };
    println!("cl_p: {:?}", *cl_p); // dereference and print entire list
    println!("head_node: {:?}", head_node);
}

// define custom smart pointer
// the parameter `T` is generic, so MyBox is a tuple with only one
// element of a generic type
struct MyBox<T> (T, String);

impl<T> MyBox<T> {
    fn new(v:T, key:&str) -> MyBox<T> {
        MyBox(v, key.to_string())
    }
}
impl<T> Deref for MyBox<T> {
    type Target = T; // TODO, is it type alias ?
    fn deref(&self) -> & Self::Target {&self.0}
    // tell callers which instance / value to reference
}
impl<T> DerefMut for MyBox<T> {
    fn deref_mut(& mut self) -> & mut Self::Target {
        &mut self.0
    } // TODO, no need to declare Target ?
}
impl<T>  Drop for MyBox<T> {
    fn drop(& mut self) {
        println!("MyBox pointer is destroying {}", self.1);
    }
}

fn  deref_coercion_test(s:&str) {
    println!("Hello, {s}!");
}

fn  deref_trait_demo()
{
    println!("-------- deref/drop trait demo --------");
    let x:u8 = 13;
    let mut y = Box::new(x); // copy value in `x` to `y`
    let mut z:MyBox<u8> = MyBox::new(x, &"cpy-cat");
    assert_eq!(13, x);
    assert_eq!(13, *y);
    assert_eq!(13, *z);
    *y += 5;
    *z  = *z * 6;
    assert_eq!(78, *z);
    *z -= 1;
    assert_eq!(13, x);
    assert_eq!(18, *y);
    assert_eq!(77, *z);
    let sp = MyBox::new(String::from("kangroo"), &"jump");
    // since standard library implements Deref trait on String type
    // , the reference to MyBox<String> type will be converted to the
    // reference to String type, then convert it to reference to string
    // literal
    deref_coercion_test(&sp);
} // end of deref_trait_demo


#[derive(Debug)]
enum ShrListNode {
    Cons(u32, Rc<ExListNode>),
    // Nil
} // Box smart pointer is applied when the size of type is

fn smart_ptr_refcnt_demo()
{ // note references to the rc pointer have to be immutable
    println!("-------- smart-pointer ref-cnt demo --------");
    // single ownership
    let shared_final  = ExListNode::Cons(72, Box::new(ExListNode::Nil));
    let shared_middle = ExListNode::Cons(30, Box::new(shared_final));
    let _shared_head = ExListNode::Cons(1884, Box::new(shared_middle));
    let shared_head  = Rc::new(_shared_head);
    // shared ownership of `shared_head` among several nodes
    let node_b = ShrListNode::Cons(510, Rc::clone(&shared_head));
    let node_c = ShrListNode::Cons(742, Rc::clone(&shared_head));
    assert_eq!(Rc::strong_count(&shared_head), 3);
    {
        let _node_d = ShrListNode::Cons(18364, Rc::clone(&shared_head));
        assert_eq!(Rc::strong_count(&shared_head), 4);
    }
    assert_eq!(Rc::strong_count(&shared_head), 3);
    println!("node_b: {:?}", node_b);
    println!("node_c: {:?}", node_c);
} // end of  smart_ptr_refcnt_demo


trait AbsMessager {
    fn send(&self, body:&str);
}
// Trait bound syntax (ch10-3, The Book) can also be applied
// to struct declaration. Here the generic type `T` is constrained
// by the types implementing only `AbsMessager` trait
struct LimitTracker<'a , T:AbsMessager>
{
    // the field below holds reference to an instance of generic
    // type `T`, lifetime should be annotated
    sender:& 'a T,
    value:usize,
    max:usize,
}
impl<'a,T> LimitTracker<'a,T>  where T: AbsMessager,
{ // `Self` is the alias covering the whole syntax
  // --> LimitTracker<'a,T:AbsMessager>
    fn new(s:&'a T, _max:usize) -> Self {
        Self {sender:s, max:_max, value:0}
    }
    fn set(&mut self, v:usize) {
        self.value = v;
        let percentage_of_max = self.value as f64 / self.max as f64;
        if percentage_of_max >= 1.0 {
            self.sender.send("quota use up");
        } else if percentage_of_max >= 0.9 {
            self.sender.send("warning, use over 90% of your quota");
        } else if percentage_of_max >= 0.75 {
            self.sender.send("use over 75%, consider to increase capacity");
        }
    }
}

struct MockMessager {
    sent_msgs:RefCell<Vec<String>>,
}
impl MockMessager {
    fn new() -> Self
    { Self{sent_msgs: RefCell::new(vec![])} }
}
impl AbsMessager for MockMessager {
    // the self variable shouldn't be mutable, to match definition in `AbsMessager` trait
    fn send(&self, body:&str) {
        // has to be mutable only for testing purpose
        self.sent_msgs.borrow_mut().push(body.to_string());
    }
}

fn smart_ptr_refcell_demo()
{
    println!("-------- smart-pointer ref-cell demo --------");
    let appmax:usize = 101;
    // the mock messager instance on caller side is immutable
    let mockmsger = MockMessager::new();
    let mut tracker = LimitTracker::new(&mockmsger, appmax);
    // without `RefCell` instance, compiler will report error cuz the following call
    // to `set()` function will make the tracker add new content to the mock messager.
    tracker.set(78);
    assert_eq!(mockmsger.sent_msgs.borrow().len(), 1);
    tracker.set(95);
    assert_eq!(mockmsger.sent_msgs.borrow().len(), 2);
    // RefCell, borrowing rules at runtime time:
    // - either mutable or immutable borrow can occur at the same time
    // - can be mutably borrowed only once
    // - can be immutably borrowed more than once
    {
        let mut b1:RefMut<Vec<String>> = mockmsger.sent_msgs.borrow_mut();
        assert_eq!(mockmsger.sent_msgs.try_borrow_mut().is_err(), true);
        assert_eq!(mockmsger.sent_msgs.try_borrow().is_err(), true);
        assert_eq!(mockmsger.sent_msgs.try_borrow().is_err(), true);
        let last_sent_content = b1.pop().unwrap();
        assert_eq!(last_sent_content, "warning, use over 90% of your quota");
        assert_eq!(b1.len(), 1);
    } // b1 goes out of scope, return the borrow
    {
        let b2:Ref<Vec<String>> = mockmsger.sent_msgs.borrow();
        let b3:Ref<Vec<String>> = mockmsger.sent_msgs.borrow();
        assert_eq!(mockmsger.sent_msgs.try_borrow_mut().is_err(), true);
        assert_eq!(b3.len(), 1);
        assert_eq!(b2[0], "use over 75%, consider to increase capacity");
        assert_eq!(b2.len(), 1);
    }
} // end of smart_ptr_refcell_demo



enum ListnodeLinkType {
    // ref-cell wraps Option and Rc/Weak type, in this case it can be borrowed
    // mutably for setting Rc instance after node declaration
    StrongLink( RefCell<Option<Rc<MLeakLLnode>>> ),
    WeakLink( RefCell<Option<Weak<MLeakLLnode>>> ),
}

struct MLeakLLnode {
    _value: u16,
    next: ListnodeLinkType,  // RefCell<Option<Rc<MLeakLLnode>>>,
}

// CAUTION, generic type MUST NOT be used to define another generic type
// in the return type, the following function will cause compile error
//  with the message like `return type mismatch`
// fn _get_llnode_link_type<T> (x:&ListnodeLinkType)
//     -> &RefCell<Option<T>> {
//     match x {
//         ListnodeLinkType::StrongLink(l) => l,
//         ListnodeLinkType::WeakLink(l) => l ,
//     }
// }

fn refcycle_memleak_demo()
{ // create linked list first
    println!("-------- reference-cycle memory-leak demo --------");
    // `b` itself does not need to be mutable
    let  b = Rc::new(MLeakLLnode{ _value:187, next:
        ListnodeLinkType::StrongLink(RefCell::new(None))
    });
    let  a = Rc::new(MLeakLLnode{ _value:254, next:
        ListnodeLinkType::StrongLink( RefCell::new(Some(Rc::clone(&b))) )
    });
    assert_eq!(Rc::strong_count(&b), 2);
    assert_eq!(Rc::strong_count(&a), 1);
    // strong reference are how you share ownership of a RC<T>
    match &b.next { // feed reference to avoid ownership movement below
        ListnodeLinkType::StrongLink(b_link) => {
            println!("link last node b to first node a");
            // setting new Rc instance below requires DerefMut trait, RefCell implements it
            // while Rc doesn't, without RefCell, compiler will report error
            *b_link.borrow_mut() = Some(Rc::clone(&a));
        },
        _others => {},
    }
    assert_eq!(Rc::strong_count(&b), 2);
    assert_eq!(Rc::strong_count(&a), 2);
    // can also access the field directly through `a` ?
    let a_ptr = Rc::as_ref(&a);
    println!("node a found");
    assert_eq!(a_ptr._value, a._value);
    assert_eq!(a_ptr._value, 254);
    match &a_ptr.next {
        ListnodeLinkType::StrongLink(a_link) => {
            if let Some(b_ptr) = & * a_link.borrow()
            { // dereference immediately followed by reference, to avoid ownership movement
                println!("node b found");
                assert_eq!(b_ptr._value, b._value);
                match &b_ptr.next {
                    ListnodeLinkType::StrongLink(b_link) => {
                         if let Some(a_dup_ptr) = &*b_link.borrow() {
                             println!("node a found AGAIN ... loop detected");
                             assert_eq!(a_dup_ptr._value, a._value);
                         }
                    },
                    _others => {},
                }
            }
        },
        _others => {},
    };
    // add extra semicolon at the end, to extend life of `a` and `b`
} // end of  refcycle_memleak_demo


fn  refcycle_avoid_leak_demo()
{
    println!("-------- reference-cycle avoid memory-leak demo --------");
    let  b = Rc::new(MLeakLLnode{ _value:187, next:
        ListnodeLinkType::WeakLink(RefCell::new(None))
    });
    { // inner scope start
        // weak references don't express an ownership relationship,
        // their counts don't affect when cleaning up Rc<T> instance
        let  a = Rc::new(MLeakLLnode{ _value:254, next:
            ListnodeLinkType::StrongLink( RefCell::new(Some(Rc::clone(&b))) )
        });
        assert_eq!(Rc::strong_count(&b), 2);
        assert_eq!(Rc::strong_count(&a), 1);
        assert_eq!(Rc::weak_count(&a), 0);
        match &b.next { // feed reference to avoid ownership movement below
            ListnodeLinkType::WeakLink(b_link) => {
                println!("link last node b to first node a");
                // setting new Rc instance below requires DerefMut trait, RefCell implements it
                // while Rc doesn't, without RefCell, compiler will report error
                *b_link.borrow_mut() = Some(Rc::downgrade(&a));
            },
            _others => {},
        }
        assert_eq!(Rc::strong_count(&b), 2);
        assert_eq!(Rc::strong_count(&a), 1);
        assert_eq!(Rc::weak_count(&a), 1);
        match &b.next {
            ListnodeLinkType::WeakLink(b_link) => {
                if let Some(a_dup) = &*b_link.borrow()
                { // to make sure MLeakLLnode instance still exists, call Weak<T>::upgrade()
                    println!("node `a` linked after `b` ... loop detected, {:?}", a_dup);
                    let a_dup = a_dup.upgrade().unwrap();
                    assert_eq!(a_dup._value, a._value);
                }
            },
            _others => {},
        };
    } // inner scope end
    assert_eq!(Rc::strong_count(&b), 1);
    match &b.next {
        ListnodeLinkType::WeakLink(b_link) => {
            if let Some(a_ptr) = &*b_link.borrow()
            {
                println!("node `a` went out of scope and shouldn be dropped");
                assert_eq!(a_ptr.upgrade().is_none(), true);
            }
        },
        _others => {},
    };
} // end of  refcycle_avoid_leak_demo

fn destructing_breakapart_demo()
{
    println!("-------- destructing to break-apart value demo --------");
    let p = Rectangle {width: 186, height:535};
    let Rectangle {width: d0, height: d1} = p;
    assert_eq!(d0, 186);
    assert_eq!(d1, 535);
}

fn  deref_raw_ptr_demo()
{
    println!("-------- dereference raw pointer demo --------");
    // avoid dereferencing a invalid pointer as much as possible
    // cuz Rust standard library does not provide any mechanism
    // for exception handling at OS level. (which implicitly means
    // you may still need to handle that in C code)
    // ----------------------------
    // let addr = 0xe234abc0u32;
    // let ptr:* const i8 = addr as * const i8;
    // *ptr // <-- segmentation fault at OS, terminated abormally
    // ----------------------------
    let mut value:u8 = 0xe2;
    let p1:*const u8 = & value as * const u8;
    let p2:*mut u8   = & mut value as * mut u8;
    unsafe {
        assert_eq!(*p1, 0xe2u8);
        *p2 -= 3;
        assert_eq!(*p1, 0xdfu8);
        *p2 -= 10;
        assert_eq!(*p1, 0xd5u8);
    } // compiler omitted the borrowing rules so mutable and immutable
      // references can co-exist in the same scope.
} // end of deref_raw_ptr_demo

fn my_split_at_mut(values:&mut[i16], mid:usize) -> (&mut[i16], &mut[i16])
{ 
    // raw pointer declared with primitive type, not vector, also no need
    // to add `mut` after `let` 
    let ptr : * mut i16 = values.as_mut_ptr();
    let totlen = values.len();
    assert!(totlen > mid);
    unsafe {
        // compiler will report error to the code below, even when it is
        // already in unsafe block ...
        // (&mut values[..mid], &mut values[mid..])
        ( 
            slice::from_raw_parts_mut(ptr, mid),
            slice::from_raw_parts_mut(ptr.add(mid), totlen - mid),
        )
    }
} // end of my_split_at_mut

fn unsafe_block_demo()
{
    println!("-------- unsafe block demo --------");
    let mut values:Vec<i16> = vec![-92, 30, 551, -47, 918, -55];
    let mid = 3;
    let (d0,d1) = my_split_at_mut(&mut values, mid);
    d0[0] += 3;
    d1[1] -= 10;
    assert_eq!(d0, [-89, 30, 551]);
    assert_eq!(d1, [-47, 908, -55]);
    println!("original values: {:?}", values);
}

use std::sync::Mutex;
// static lifetime
static mut GLOBAL_COUNTER : i32 = 15;
static GLOBAL_COUNTER_2: Mutex<Cell<Rectangle>> = 
    Mutex::new(Cell::new(
        Rectangle{width:100u16, height:125u16}
    ));

// app developers are responsible to ensure that gloval variables
// are accessed in thread-safe situations, such as put them in
// `unsafe` block or use it within `Mutex`, otherwise compiler
// will report error.
fn  global_var_demo()
{
    println!("-------- global variable demo --------");
    unsafe {
        GLOBAL_COUNTER += 4;
        let newval = GLOBAL_COUNTER;
        assert_eq!(newval, 19i32);
    }
    if let Ok(mut currcell) = GLOBAL_COUNTER_2.lock() {
        let mut value = currcell.get_mut();
        value.width += 7;
        value.height = value.height >> 1;
    }
    if let Ok(mut currcell) = GLOBAL_COUNTER_2.lock() {
        // request the reference without moving the ownership
        let value = currcell.get_mut();
        assert_eq!(value.width, 107u16);
        println!("new value in GLOBAL_COUNTER_2: {:?}", value);
    }
}


struct Vertex2D{x:i32, y:i32}

// Vertex3D declared above
// set default value to generic type parameter in Add<T>
impl Add<Vertex2D> for Vertex3D {
    type Output = Vertex3D;
    fn add(self, n:Vertex2D) -> Self::Output {
        Vertex3D(
            self.0 + (n.x as f32),
            self.1 + (n.y as f32),
            self.2 - 0.1,
        )
    }
}
fn overload_operator_demo()
{
    println!("-------- overload operator demo --------");
    let p0 = Vertex3D(2.3, 4.5, 6.7) ;
    let p1 = Vertex2D{x:365,y:-22};
    let p2:Vertex3D = p0 + p1;
    println!("p2 is {:?}", p2);
}

trait WolfSpider {fn hunt(&self);}
trait TigerShark {fn hunt(&self);}
struct Hunter;
impl WolfSpider for Hunter {
    fn hunt(&self) {
        println!("the web trapping insects then poison them");
    }
}
impl TigerShark for Hunter {
    fn hunt(&self) {
        println!("swim fast toward the target then bite");
    }
}
impl Hunter {
    fn hunt(&self) {
        println!("capture then kill the target");
    }
}

fn call_method_disambiguation_demo()
{
    println!("-------- call-method disambiguation demo --------");
    let h = Hunter{};
    h.hunt();
    WolfSpider::hunt(&h);
    TigerShark::hunt(&h);
}

fn declarative_macro_demo()
{ // define a local macro
    macro_rules! my_ez_vec {
        // the order of the arms determines which code block
        // will be generated first
        ( $y:ident ) => { // `ident` is the variable declared previously
            {
                let mut out = Vec::new();
                $y(&mut out);
                out
            }
        };
        ( $($x:literal),* ) => { // `literal` has to be integer or string or byte
            { // this arm creates inner scope of code block which
              // returns vector instance at the end
                let mut out = Vec::new();
                $( out.push($x); )*
                out
            }
        }; // the bracket here is referred to by macro declaration
    }
    println!("-------- declarative macro demo --------");
    let lst1 =  my_ez_vec![56, -439, 17];
    let lst2 =  my_ez_vec![5.6, -4.39, 1.7];
    let c = |outp:&mut Vec<String>| {
        outp.push("chatGPT".to_string());
        outp.push("MsBing".to_string());
    };
    let lst3:Vec<String> = my_ez_vec!(c);
    assert_eq!(lst1, [56, -439, 17]);
    assert_eq!(lst2, [5.6, -4.39, 1.7]);
    assert_eq!(lst3, ["chatGPT","MsBing"]);
    println!("lst3 : {:?}", lst3);
} // end of declarative_macro_demo


fn main() {
    owner_ref_demo();
    struct_demo();
    enum_demo();
    vector_demo();
    string_demo();
    hashmap_demo();
    recoverable_error_demo();
    generic_func_demo();
    trait_demo();
    lifetime_variable_demo();
    functional_closure_demo();
    functional_iterator_demo();
    smart_ptr_box_demo();
    deref_trait_demo();
    smart_ptr_refcnt_demo();
    smart_ptr_refcell_demo();
    // the demo below causes memory leak, run with Valgrind to see the report
    refcycle_memleak_demo();
    // the demo below shows how to create cycle without memory leak
    refcycle_avoid_leak_demo();
    destructing_breakapart_demo();
    deref_raw_ptr_demo();
    unsafe_block_demo();
    global_var_demo();
    overload_operator_demo();
    call_method_disambiguation_demo();
    declarative_macro_demo();
} // end of main
