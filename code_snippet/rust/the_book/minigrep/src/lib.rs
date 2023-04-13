use std::error::Error as StdError;
use std::{env, fs};
use std::io::{self, BufReader, BufRead};

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    // the first argument can be any type which implements `Iterator` trait
    pub fn build (mut args :impl Iterator<Item=String>) -> Result<Self, & 'static str>
    {
        // set static lifetime to error string literal, make it
        // available as long as program runs
        args.next(); // discard, it should be the path to executable program
        let _qry   : String = match args.next() {
            Some(s) => if s.len() == 0 {return Err("empty query string");} else {s},
            None => return Err("not enough arguments"),
        };
        let _fpath : String = match args.next() {
            Some(s) => if s.len() == 0 {return Err("empty file path for search");} else {s},
            None => return Err("not enough arguments"),
        };
        let  ign_ks = env::var("IGNORE_CASE").is_ok();
        let _qry : String = if ign_ks {_qry.to_lowercase()} else {_qry};
        Ok(Self{query:_qry, file_path:_fpath, ignore_case:ign_ks, })
        // return based on the error message
    }
} // end of impl config

fn chk_one_line (line:& str, keyword:& str, l_ign_ks:bool) -> bool
{
    if line.is_empty() || keyword.is_empty() {
        false
    } else {
        if l_ign_ks {
            let l:String = line.to_lowercase();
            let l:&str = &l.as_str();
            l.contains(keyword)
        } else {
            line.contains(keyword)
        }
    }
}


fn search_and_print (_file:fs::File, cfg:&Config,
                     _linechk:fn(&str, &str, bool) -> bool ) -> u32
{ // read portion of file instead of entire content
    let rdbuf = BufReader::new(_file);
    // `BufReader::lines()`
    // - return an iterator with items of type `Result<String, io::Error>`
    // - move ownership of the receiver (the BufReader instance)
    let all_lines = rdbuf.lines();
    // the closure has to capture struct fields in the same scope,
    // so it cannot be converted to a function
    let f = |line:&Result<String, io::Error>| -> bool {
        match line {
            Ok(s) => _linechk( &s.as_str(),
                &cfg.query.as_str(), cfg.ignore_case),
            Err(_) => false,
        }
    };
    let mut num:u32 = 0;
    for line in all_lines.filter(f) {
        println!("found line : {:?}", line);
        num += 1;
    }
    num
} // end of search_and_print

pub fn run(cfg:&Config) -> Result<(), Box<dyn StdError>>
{
    let f = fs::File::open(cfg.file_path.as_str()) ?;
    let num = search_and_print(f, cfg, chk_one_line);
    println!("num of lines found : {num}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::chk_one_line;

    #[test]
    fn build_cfg_ok() {
        let t_args = [
            "/path/to/myprogram".to_string(),
            "fabric".to_string(),
            "/path/to/file".to_string()
        ];
        let actual_cfg = Config::build(t_args.into_iter());
        assert_eq!(actual_cfg.is_ok(), true);
        let actual_cfg = actual_cfg.unwrap();
        assert_eq!(actual_cfg.file_path, "/path/to/file");
        assert_eq!(actual_cfg.query, "fabric");
    }

    #[test]
    fn build_cfg_missing_arg() {
        let t_args = [
            "/path/to/myprogram".to_string(),
            "fabric".to_string(),
        ];
        let actual_cfg = Config::build(t_args.into_iter());
        assert_eq!(actual_cfg.is_err(), true);
    }

    #[test]
    fn build_cfg_empty_filepath() {
        let t_args = [
            "/path/to/myprogram".to_string(),
            "fabric".to_string(),
            "".to_string()
        ];
        let actual_cfg = Config::build(t_args.into_iter());
        assert_eq!(actual_cfg.is_err(), true);
    }

    #[test]
    fn saerch_missing_arg() {
        let result = chk_one_line(&"", &"", false);
        assert_eq!(result, false);
        let result = chk_one_line(&"abcde", &"", false);
        assert_eq!(result, false);
        let result = chk_one_line(&"", &"abcde", true);
        assert_eq!(result, false);
    }

    #[test]
    fn saerch_found() {
        let result = chk_one_line(&"temp probe;closure", &"pro", false);
        assert_eq!(result, true);
        let result = chk_one_line(&"one-soLution", &"lut", false);
        assert_eq!(result, false);
        let result = chk_one_line(&"free-sEcUre\n", &"secu", true);
        assert_eq!(result, true);
        let result = chk_one_line(&"free-sEcUre\n", &"secu", false);
        assert_eq!(result, false);
    }
} // end of tests module

