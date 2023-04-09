use std::error::Error as StdError;
use std::{env, fs};

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    pub fn build (args:&[String]) -> Result<Self, & 'static str> {
        // set static lifetime to error string literal, make it
        // available as long as program runs
        let mut errmsg:Option<&str> = None;
        if args.len() < 3 {
            errmsg = Some("not enough arguments");
        } else if args[1].len() == 0 {
            errmsg = Some("empty query string");
        } else if args[2].len() == 0 {
            errmsg = Some("empty file path for search");
        }
        match errmsg {
            Some(detail) => Err(detail),
            None => {
                let _qry   = args[1].clone();
                let _fpath = args[2].clone();
                let  ign_ks = env::var("IGNORE_CASE").is_ok();
                Ok(Self{query:_qry, file_path:_fpath,
                    ignore_case:ign_ks, })
            },
        } // return based on the error message
    }
} // end of impl config

fn search<'a>(fulltxt: & 'a str, keyword:& str, ign_ks:bool) -> Vec<& 'a str>
{ // the lifetime of each result must be the same as fcontent
    let mut out:Vec<&str> = vec![];
    if fulltxt.is_empty() || keyword.is_empty() {
    } else {
        // to-lowercase() will generate new value and allocate new space
        // it is safe to claim the varaible for appropriate lifetime
        let kw:String = keyword.to_lowercase();
        let kw:&str = & kw.as_str();
        for line in fulltxt.lines() {
            let pattern_exists:bool = 
                if ign_ks {
                    let l:String = line.to_lowercase();
                    let l:&str = &l.as_str();
                    l.contains(kw)
                } else {line.contains(keyword)};
            if pattern_exists {
                out.push(line);
            }
        } // end of loop
    }
    out
} // end of search

pub fn run(cfg:&Config) -> Result<(), Box<dyn StdError>>
{ // TODO, read portion of file instead of entire content
    let fcontent = fs::read_to_string(cfg.file_path.as_str()) ?;
    for line in search(&fcontent, &cfg.query, cfg.ignore_case) {
        println!("found line : {}", line);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Config;
    use crate::search;

    #[test]
    fn build_cfg_ok() {
        let t_args = [
            "/path/to/myprogram".to_string(),
            "fabric".to_string(),
            "/path/to/file".to_string()
        ];
        let actual_cfg = Config::build(&t_args);
        assert_eq!(actual_cfg.is_ok(), true);
        assert_eq!(actual_cfg.unwrap().file_path, "/path/to/file");
    }

    #[test]
    fn build_cfg_missing_arg() {
        let t_args = [
            "/path/to/myprogram".to_string(),
            "fabric".to_string(),
        ];
        let actual_cfg = Config::build(&t_args);
        assert_eq!(actual_cfg.is_err(), true);
    }

    #[test]
    fn build_cfg_empty_filepath() {
        let t_args = [
            "/path/to/myprogram".to_string(),
            "fabric".to_string(),
            "".to_string()
        ];
        let actual_cfg = Config::build(&t_args);
        assert_eq!(actual_cfg.is_err(), true);
    }

    #[test]
    fn saerch_missing_arg() {
        let result:Vec<&str> = search(&"", &"", false);
        assert_eq!(result.len(), 0);
        let result:Vec<&str> = search(&"abcde", &"", false);
        assert_eq!(result.len(), 0);
        let result:Vec<&str> = search(&"", &"abcde", true);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn saerch_found() {
        let result:Vec<&str> = search(&"temp probe\nban prohibit\nclosure\n", &"pro", false);
        assert_eq!(result.len(), 2);
        let result:Vec<&str> = search(&"one-liner\nsolution\nwhich", &"lut", false);
        assert_eq!(result.len(), 1);
        let result:Vec<&str> = search(&"eagle-fly\nfree-secure\nHoseClimb\nbasecamp\n", &"sec", false);
        assert_eq!(result.len(), 2);
        let result:Vec<&str> = search(&"eagle-fly\nfree-secure\nHoseClimb\nbasecamp\n", &"sec", true);
        assert_eq!(result.len(), 3);
        let result:Vec<&str> = search(&"sure\n5ur3\nSuRe\nbingo\nsUrE\n", &"URE", true);
        assert_eq!(result.len(), 3);
    }
} // end of tests module

