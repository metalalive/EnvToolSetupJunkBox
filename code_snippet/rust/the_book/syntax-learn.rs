use std::collections::HashMap;
use std::fs::File as LocalFile;
use std::io::{ErrorKind, Error as IOError};
use std::cmp::PartialOrd; // a trait which implements compare operator e.g. >, <, >=
use std::fmt::{Display, Formatter, Result as FmtResult};
// use std::marker::Sized;

fn owner_ref_demo() {
    println!("-------- ownership and reference --------");
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
}


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
    }
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
    match cmd_in {
        MqttCmdMsg::Publish {content, qos} => {
            let slen = content.len() as i8;
            let q = qos as i8;
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
{
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

// `?Sized` is used fo ignoring the bound to `Sized` trait, to allow the size
// of the arguement `item`  to be unknown.
fn notify_article<T:ArticleSummary + Display + ? Sized> (item:&T)
{ // the function cannot know size of the input type implementing the traits
  // at compile time, however that should be ok in this case cuz the input
  // is a reference
    println!("notifying article : {}", item.to_string());
}

fn trait_demo() {
    println!("-------- trait demo --------");
    // after generating a new instance, the function returns points to dynamic types
    // which implement the same trait.
    // The syntax `*` is dereference from the Box instance to original instance.
    // However, the instance size is unknown, that will cause compile error.
    // The alternative is to convert them to references.
    let onetweeet = &* caller_generate_article(true);
    let report    = &* caller_generate_article(false);
    notify_article(onetweeet);
    notify_article(report);
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
} // end of main

