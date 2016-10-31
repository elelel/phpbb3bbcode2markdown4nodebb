extern crate regex;
extern crate marksman_escape;

mod state;
mod tags;

use bbcode::state::State;
use bbcode::tags::parse_tag;
pub use bbcode::tags::KNOWN_TAGS;
use self::regex::Regex;
use self::marksman_escape::{Unescape};

use std::fs::OpenOptions;
use std::io::Write;

fn convert_quote(state: &mut State, nickname: &str) {
    if state.quote_level == 0 {
        state.append_end_line();
    }
    if nickname != "" {
        state.quote_level = state.quote_level + 1;
        let re_quotes = Regex::new(r"&quot;").unwrap();
        let nick_wo_quotes = re_quotes.replace_all(nickname, "");
        let re_spaces = Regex::new(r"\s").unwrap();
        let nick_wo_spaces = re_spaces.replace(&*nick_wo_quotes, "-");
        let nick = "@".to_string() + &*nick_wo_spaces.to_string() + ": ";
        state.append_start_line();
        state.append(&*nick);
        state.append_end_line();
        state.append_start_line();
    };
    state.quote_level = state.quote_level + 1;
    state.skip_while_tag_not_closed();
}

fn convert_quote_close(state: &mut State) {
    if state.quote_level == 0 {
        println!("BBCode parser: probably superfluous /quote tag");
    } else {
        state.quote_level = state.quote_level - 1;
        if state.quote_level == 0 {
            state.append_end_line();
        }
    }
    state.skip_while_tag_not_closed();
}

fn convert_bold(state: &mut State) {
    state.strong_level = state.strong_level + 1;
    state.append_strong();
    state.skip_while_tag_not_closed();
}

fn convert_bold_close(state: &mut State) {
    if state.strong_level == 0 {
        println!("BBCode parser: probably superfluous /b tag");
    } else {
        state.strong_level = state.strong_level - 1;
        state.append_strong();
    }
    state.skip_while_tag_not_closed();
}

fn convert_itallic(state: &mut State) {
    state.emphasis_level = state.emphasis_level + 1;
    state.append_emphasis();
    state.skip_while_tag_not_closed();
}

fn convert_itallic_close(state: &mut State) {
    if state.emphasis_level == 0 {
        println!("BBCode parser: probably superfluous /i tag");
    } else {
        state.append_emphasis();
        state.emphasis_level = state.emphasis_level - 1;
    }
    state.skip_while_tag_not_closed();
}

fn convert_url(state: &mut State, url: &str) {
    if url != "" {
        state.url = String::from_utf8(Unescape::new(url.bytes()).collect()).unwrap();
    } else {
        state.url = "http:://link".to_string();
    }
    state.append_url_start();
    state.skip_while_tag_not_closed();
}

fn convert_url_close(state: &mut State) {
    state.append_url_end();
    state.skip_while_tag_not_closed();
}

fn convert_img(state: &mut State, img_url: &str) {
    state.img_url = img_url.to_owned();
    if &*img_url == "" {
        state.skip_while_tag_not_closed();
        let img_url = state.take_until_tag_open().unwrap().to_owned();
        state.img_url = String::from_utf8(Unescape::new(img_url.bytes()).collect()).unwrap();
        //println!("Pushing intag img {}", &*state.img_url);
        state.waiting_for_img_close = true;
    } else {
        //println!("Appending value img {}", &*state.img_url);
        state.waiting_for_img_close = false;
        state.append_img();
        state.skip_while_tag_not_closed();
    }
}

fn convert_img_close(state: &mut State) {
    if state.waiting_for_img_close {
        //println!("Appending pushed intag img {}", &*state.img_url);
        state.append_img();
    }
    state.skip_while_tag_not_closed();
}

fn convert_size(state: &mut State) {
    state.skip_while_tag_not_closed();
}

fn convert_size_close(state: &mut State) {
    state.skip_while_tag_not_closed();
}

fn convert_color(state: &mut State, color: &str) {
    if color != "" {
        state.color_stack.push(color.to_owned());
    } else {
        state.color_stack.push("black".to_owned());
    }
    state.append_color_start();
    state.skip_while_tag_not_closed();
}

fn convert_color_close(state: &mut State) {
    if state.color_stack.len() == 0 {
        println!("BBCode parser: probably superfluous /color tag");
    } else {
        state.append_color_end();
        state.color_stack.pop();
    }
    state.skip_while_tag_not_closed();
}

fn convert_list(state: &mut State) {
    println!("BBCode parser: no converter for list tag yet");
    state.move_next_char_to_output();
}

fn convert_list_item(state: &mut State) {
    println!("BBCode parser: no converter for list item tag yet");
    state.move_next_char_to_output();
}

fn convert_list_close(state: &mut State) {
    println!("BBCode parser: no converter for /list tag yet");
    state.move_next_char_to_output();
}
fn convert_email(state: &mut State) {
    println!("BBCode parser: no converter for email tag yet");
    state.move_next_char_to_output();
}
fn convert_email_close(state: &mut State) {
    println!("BBCode parser: no converter for /email tag yet");
    state.move_next_char_to_output();
}
fn convert_flash(state: &mut State) {
    println!("BBCode parser: no converter for flash tag yet");
    state.move_next_char_to_output();
}
fn convert_flash_close(state: &mut State) {
    println!("BBCode parser: no converter for /flash tag yet");
    state.move_next_char_to_output();
}

fn convert_attachment(state: &mut State, maybe_phpbb_tid: Option<u32>) {
    println!("Convertin attachment");
    match maybe_phpbb_tid {
        Some(phpbb_tid) => {
            state.skip_while_tag_not_closed();
            let intag = state.take_until_tag_open().unwrap().to_owned();
            let re = Regex::new(r"(?is)<!-- ia. -->(.*?)<!-- ia. -->").unwrap();
            match re.captures_iter(&*intag).next() {
                Some(x) => {
                    let filename = "attached_image.txt";
                    let url = "/attached_images/".to_owned() + &*phpbb_tid.to_string() + "_" + x.at(1).unwrap();
                    let mut file = OpenOptions::new().write(true).append(true).open(filename);
                    match file {
                        Ok(_) => {};
                        Error(error) => {
                            if error.code == 2 {
                                file = OpenOptions::new().write(true).append(true).create(filename);
                            } else {
                                panic!("Unhandled error {} while opening attached images log", error);
                            }
                        }
                    }
                    if let Err(e) = file.write_fmt(format_args!("{}\n", url)) {
                        println!("Error writing attachment image log {}", e);
                    }
                    state.img_url = url.to_owned();
                    state.img_alt_text = "Attached image ".to_owned() + x.at(1).unwrap();
                    println!("BBCode parser: pointing attached image url to {}", &*state.img_url);
                    state.waiting_for_img_close = false;
                    state.append_img();
                }
                None => {
                    println!("BBCode parser: no converter for this attachment tag yet: {}", &*intag);
                }
            }
        },
        None => {
            panic!("Can't generate image attachemtn url because phpbb topic id is not found")
        }
    }
}

fn convert_attachment_close(state: &mut State) {
    state.skip_while_tag_not_closed();
}

fn convert_spoiler(state: &mut State) {
    state.spoiler_level = state.spoiler_level + 1;
    state.append_spoiler();
    state.skip_while_tag_not_closed();
}

fn convert_spoiler_close(state: &mut State) {
    if state.spoiler_level == 0 {
        println!("BBCode parser: probably superfluous /spoiler tag");
    } else {
        state.spoiler_level = state.spoiler_level - 1;
    }
    state.skip_while_tag_not_closed();
}

fn parse_line(phpbb_tid: Option<u32>, state: &mut State, s: &str) {
    state.input = s.to_owned();
    loop {
        state.append_until_tag_open(); 
//        println!("Processing input: {}", state.input);
//        println!("Processing output: {}", state.output);
        if &*state.input != "" {
            let t = parse_tag(state);
            match t {
                Some(tag) => {
                    match &*tag.name {
                        "quote" => convert_quote(state, &*tag.value),
                        "/quote" => convert_quote_close(state),
                        "b" => convert_bold(state),
                        "/b" => convert_bold_close(state),
                        "i" => convert_itallic(state),
                        "/i" => convert_itallic_close(state),
                        "url" => convert_url(state, &*tag.value),
                        "/url" => convert_url_close(state),
                        "img" => convert_img(state, &*tag.value),
                        "/img" => convert_img_close(state),
                        "size" => convert_size(state),
                        "/size" => convert_size_close(state),
                        "color" => convert_color(state, &*tag.value),
                        "/color" => convert_color_close(state),
                        "u" => convert_bold(state),
                        "/u" => convert_bold_close(state),
                        "list" => convert_list(state),
                        "*" => convert_list_item(state),
                        "/list" => convert_list_close(state),
                        "email" => convert_email(state),
                        "/email" => convert_email_close(state),
                        "flash" => convert_flash(state),
                        "/flash" => convert_flash_close(state),
                        "attachment" => convert_attachment(state, phpbb_tid),
                        "/attachment" => convert_attachment_close(state),
                        "spoiler" => convert_spoiler(state),
                        "/spoiler" => convert_spoiler_close(state),
                        _ => {
                            // Unknown tag
                            println!("Unknown tag: {}", &*tag.name);
                            state.move_next_char_to_output();
                        }
                    }
                }
                None => {
                    // Not a tag
                    // println!("BBCode parser: Not a tag");
                    state.move_next_char_to_output();
                }
            }
        } else {
            break;
        }
    }
}

pub fn parse_text(phpbb_tid: Option<u32>, s: &str) -> String {
    let mut state = State::new();
    let split_lines_re = Regex::new(r"\n").unwrap();
    for line in split_lines_re.split(s) {
        state.append_start_line();
        parse_line(phpbb_tid, &mut state, line);
        state.append_end_line();
    }
    state.output.to_owned()        
}

#[test]
fn parse_text_test() {
    assert_eq!(&*parse_text(
        r"This is simple text.
New line.
[quote=&quot;Nickname1&quot;:3oz8flg1]Quoted text, single level[/quote:3oz8flg1]"),
        r"This is simple text.
New line.

> @Nickname1: 
> Quoted text, single level
");
    assert_eq!(&*parse_text(
        r"[b]Bold[/b] text
[i]Itallic[/i] text
[b][i]Bold Itallic[/i][/b] text
[b][i]Bold Itallic bad nesting[/b][/i] text"),
        r"*Bold* text
**Itallic** text
***Bold Itallic*** text
***Bold Itallic bad nesting*** text
");
    assert_eq!(&*parse_text(
        r"[url=http&#58;//host&#46;domain&#46;tld/file:123i2nm3]Link to http://host.domain.tld/file[/url:123i2nm3] Text on the same line as URL
Text again
"),
               r"[Link to http://host.domain.tld/file](http://host.domain.tld/file) Text on the same line as URL
Text again
");
    assert_eq!(&*parse_text(r"Img tag value variant [img=http://url1.com] text"),
        r"Img tag value variant ![](http://url1.com) text
");

assert_eq!(&*parse_text(r"Img in tag variant [img]http://url2.com[/img] text"),
           r"Img in tag variant ![](http://url2.com) text
");
}
