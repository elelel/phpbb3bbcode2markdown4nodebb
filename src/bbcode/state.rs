extern crate regex;

use self::regex::Regex;

#[derive(Eq, PartialEq, Debug)]
pub struct State {
    pub input: String,
    pub output: String,
    pub quote_level: u32,
    pub strong_level: u32,
    pub spoiler_level: u32,
    pub emphasis_level: u32,
    pub color_stack: Vec<String>,
    pub url: String,
    pub url_opened: bool,
    pub img_url: String,
    pub img_alt_text: String,
    pub waiting_for_img_close: bool
        
}

impl State {
    pub fn new() -> State {
        State {
            input: "".to_string(),
            output: "".to_string(),
            quote_level: 0,
            strong_level: 0,
            emphasis_level: 0,
            spoiler_level: 0,
            color_stack: Vec::new(),
            url: "".to_string(),
            url_opened: false,
            img_url: "".to_string(),
            img_alt_text: "".to_string(),
            waiting_for_img_close: false
        }
    }

    /*
    pub fn is_at_tag_open(&mut self) -> bool {
        let re = Regex::new(r"(?s)^\[.*").unwrap();
        let rslt = re.is_match(&*self.input);
        rslt
    }*/
    
    pub fn move_next_char_to_output(&mut self) {
        if &*self.input != "" {
            let re = Regex::new(r"(?s)^(.)(.*)").unwrap();
            match re.captures_iter(&*self.input.to_owned()).next() {
                Some(x) => {
                    self.set_input(x.at(2).unwrap_or(""));
                    self.append(x.at(1).unwrap());
                }
                None => {
                    println!("Input: {}", &*self.input);
                    panic!("move_next_char to output end of input?");
                }
            }
        } else {
        }
    }

    pub fn append_emphasis(&mut self) {
        self.output.push_str("**");
    }

    pub fn append_strong(&mut self) {
        self.output.push_str("*");
    }

    pub fn append_spoiler(&mut self) {
        self.output.push_str(" >! ");
    }

    pub fn append_url_start(&mut self) {
        self.append("[");
        self.url_opened = true;
    }
    
    pub fn append_url_end(&mut self) {
        self.output.push_str("](");
        if &*self.url == "" {
            self.url = "link".to_string();
        }
        self.output.push_str(&*self.url);
        self.output.push_str(")");
        self.url = "".to_string();
        self.url_opened = false;
    }

    pub fn append_img(&mut self) {
        if self.url_opened {
            self.append_url_end();
        }
        self.output.push_str("![");
        self.output.push_str(&*self.img_alt_text);
        self.append("](");
        self.output.push_str(&*self.img_url);
        self.append(")");
        self.img_url = "".to_string();
        self.img_alt_text = "".to_string();
    }

    pub fn append_color_start(&mut self) {
        self.output.push_str("%(");
        self.output.push_str(&*self.color_stack[self.color_stack.len() - 1]);
        self.output.push_str(")[");
    }

    pub fn append_color_end(&mut self) {
        self.output.push_str("]");
    }

    pub fn append_start_line(&mut self) {
        for _ in 0..self.quote_level {
            self.output.push_str("> ");
        }
        if self.spoiler_level > 0 { self.append_spoiler() };
        if self.emphasis_level > 0 { self.append_emphasis() };
        if self.strong_level > 0 { self.append_strong() };
        if self.color_stack.len() > 0 { self.append_color_start(); };
    }

    pub fn append(&mut self, s: &str) {
        self.output.push_str(s);
    }

    pub fn append_end_line(&mut self) {
        if self.color_stack.len() > 0 { self.append_color_end(); };
        if self.emphasis_level > 0 { self.append_emphasis() };
        if self.strong_level > 0 { self.append_strong() };
        self.output = self.output.to_owned() + "\n";
    }

    pub fn set_input(&mut self, s: &str) {
        self.input = s.to_owned();
    }

    pub fn take_until_tag_open(&mut self) -> Option<String> {
        let re = Regex::new(r"(?s)^(.*?)(\[.*)").unwrap();
        match re.captures_iter(&*self.input.to_owned()).next() {
            Some(x) => {
                self.set_input(x.at(2).unwrap_or(""));
                Some(x.at(1).unwrap().to_string())
            }
            None => None
        }
    }

    pub fn append_until_tag_open(&mut self) {
        let taken = self.take_until_tag_open();
        match taken {
            Some(x) => self.append(&*x),
            None => {
                let s = self.input.to_string();
                self.append(&s);
                self.set_input("");
            }
        }
    }

    pub fn skip_while_tag_not_closed(&mut self) {
        let re = Regex::new(r"(?s)^(.*?)\](.*)").unwrap();
        match re.captures_iter(&*self.input.to_owned()).next() {
            Some(x) => {
                self.set_input(x.at(2).unwrap_or(""));
            }
            None => {
                self.set_input("");
            }
        }    
    }
}

#[test]
fn append_until_tag_open_test() {
    let mut state = State::new();
    state.input = "abc [quote][/quote]".to_string();
    state.append_until_tag_open();
    assert_eq!(&*state.output, "abc ");
    assert_eq!(&*state.input, "[quote][/quote]");

    let mut state1 = State::new();
    state1.input = r" [quote=&quot;Nickname&quot;:3oz8flgq]text[/quote:3oz8flgq]
second line".to_string();

    state1.append_until_tag_open();
    assert_eq!(&*state1.input, r"[quote=&quot;Nickname&quot;:3oz8flgq]text[/quote:3oz8flgq]
second line");
    assert_eq!(&*state1.output, r" ");
}

#[test]
fn skip_while_tag_not_closed_test() {
    let mut state = State::new();
    state.input = "[quote]khkjk[/quote]zzz".to_string();
    state.skip_while_tag_not_closed();
    assert_eq!(&*state.input, "khkjk[/quote]zzz");
}
