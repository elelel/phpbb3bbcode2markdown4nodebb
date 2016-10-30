extern crate regex;

use self::regex::Regex;
use std::fmt;
use std::collections::HashSet;
use std::ascii::AsciiExt;
use bbcode::state::State;

#[derive(Eq, PartialEq, Debug)]
pub struct BBTag {
    pub name : String,
    pub value : String,
    pub attributes : Vec<String>,
}

impl fmt::Display for BBTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut need_comma = false;
        let mut attrs = "".to_string();
        for a in self.attributes.to_owned() {
            if need_comma { attrs.push_str(", "); };
            need_comma = true;
            attrs.push_str(&*a);
        }
        write!(f, "name: {}, value: {}, attirubtes: {}", self.name, self.value, attrs)
    }
}

lazy_static! {
    pub static ref KNOWN_TAGS: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("quote");
        s.insert("b");
        s.insert("i");
        s.insert("url");
        s.insert("img");
        s.insert("size");
        s.insert("color");
        s.insert("u");
        s.insert("list");
        s.insert("email");
        s.insert("flash");
        s.insert("attachment");
        s.insert("spoiler");
        s
    };
    pub static ref KNOWN_TAGS_CLOSE: HashSet<&'static str> = {
        let mut s = HashSet::new();
        s.insert("/quote");
        s.insert("/b");
        s.insert("/i");
        s.insert("/url");
        s.insert("/img");
        s.insert("/size");
        s.insert("/color");
        s.insert("/u");
        s.insert("/list");
        s.insert("/email");
        s.insert("/flash");
        s.insert("/attachment");
        s.insert("/spoiler");
        s
    };
}


fn trim(s: &str) -> String {
    let re = Regex::new(r"(?is)^(\s*)(.*?)(\s*)$").unwrap();
    match re.captures_iter(s).next() {
        Some(x) => { x.at(2).unwrap_or(s).to_owned() }
        None => { s.to_owned() }
    }
}

fn get_tag_name(s: &State) -> Option<String> {
    let re = Regex::new(r"\[(?is)(.*?)[\]=:](.*)$").unwrap();
    match re.captures_iter(&*s.input).next() {
        Some(x) => {
            let tag_name = trim(&*x.at(1).unwrap_or("").to_ascii_lowercase());
            match KNOWN_TAGS.contains(&*tag_name) |
                   KNOWN_TAGS_CLOSE.contains(&*tag_name) {
                true => Some(tag_name),
                _ => None
            }
        }
        None => {
            None
        }
    }
}

#[test]
fn get_tag_name_test() {
    let mut state1 = State::new();
    state1.input = "[quote][/quote]".to_string();
    let mut state2 = State::new();
    state2.input = "[quote=Nickname][/quote]".to_string();
    let mut state3 = State::new();
    state3.input = "[quote:12345678][/quote]".to_string();
    assert_eq!(get_tag_name(&state1).unwrap_or("".to_owned()), "quote");
    assert_eq!(get_tag_name(&state2).unwrap_or("".to_owned()), "quote");
    assert_eq!(get_tag_name(&state3).unwrap_or("".to_owned()), "quote");
}

fn get_tag_value(s: &State, tag_name: &str) -> String {
    let mut separ_re = r"=(.*?)[\]=:].*".to_owned();
    if tag_name == "img" {
        separ_re = r"=(.*?)[\]=].*".to_owned();
    }
    if tag_name != "" {
        let rs = r"\[(?is)".to_owned() + &*tag_name + &*separ_re;
        let re = Regex::new(&*rs).unwrap();
        match re.captures_iter(&*s.input).next() {
            Some(x) => {
                trim(x.at(1).unwrap_or(""))
            }
            None => {
                "".to_string()
            }
        }
    } else {
        "".to_string()
    }
}

#[test]
fn get_tag_value_test() {
    let mut state1 = State::new();
    state1.input = "[quote][/quote]".to_string();
    let mut state2 = State::new();
    state2.input = "[quote=Nickname][/quote]".to_string();
    let mut state3 = State::new();
    state3.input = "[quote=Nickname:dfajkfs][/quote]".to_string();
    assert_eq!(&*get_tag_value(&state1, "quote"), "");
    assert_eq!(&*get_tag_value(&state2, "quote"), "Nickname");
    assert_eq!(&*get_tag_value(&state3, "quote"), "Nickname");
}

fn get_tag_attributes(s: &State) -> Vec<String> {
    let re = Regex::new(r".*?:(.*?)[\]=].*").unwrap();
    let mut v = Vec::new();
    match re.captures_iter(&*s.input).next() {
        Some(x) => {
            let split_re = Regex::new(r";").unwrap();
            for attrib in split_re.split(x.at(1).unwrap()) {
                v.push(trim(attrib));
            }
            v
        }
        None => v
    }
}

#[test]
fn get_tag_attributes_test() {
    let mut state1 = State::new();
    state1.input = "[quote=Nickname:12345678; 2132; 01jhsdf][/quote]".to_string();
    assert_eq!(&*get_tag_attributes(&state1), ["12345678",
                                               "2132",
                                               "01jhsdf"]);
}

pub fn parse_tag(s: &State, ) -> Option<BBTag> {
    match get_tag_name(s) {
        Some(x) => {
            let val = get_tag_value(s, &*x);
            Some(BBTag {
                name: x,
                value: val,
                attributes: get_tag_attributes(s),
            })
        }
        None => None
    }
}

#[test]
fn parse_tag_test() {
    let mut state1 = State::new();
    state1.input = "[quote=Nickname:12345678; 2132; 01jhsdf][/quote]".to_string();
    let expected =  BBTag {
        name: "quote".to_string(),
        value: "Nickname".to_string(),
        attributes: vec!["12345678".to_string(),
                         "2132".to_string(),
                         "01jhsdf".to_string()]
    };
    assert_eq!(parse_tag(&state1).unwrap(), expected);
}
