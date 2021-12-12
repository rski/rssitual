use chrono::prelude::*;
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use rss::{Channel, Item};

#[derive(Clone, Debug, PartialEq)]
enum State {
    Read,
    Unread,
}

#[derive(Debug)]
struct Link {
    url: String,
    tags: Vec<String>,
    state: State,
    date: String,
}

impl Link {
    fn fetch_title(&self) -> Option<String> {
        if self.url.ends_with(".pdf") {
            return Some(self.url.to_owned());
        }
        eprintln!("fetching {}...", self.url);
        let r = reqwest::blocking::get(&self.url);
        match r {
            Err(e) => {
                eprintln!("error fetching {}: {}", self.url, e);
                None
            }
            Ok(r) => {
                if !r.status().is_success() {
                    eprintln!("error fetching {}: {}", self.url, r.status());
                    return None;
                }
                match select::document::Document::from_read(r) {
                    Err(e) => {
                        eprintln!("error decoding {}: {}", self.url, e);
                        None
                    }
                    Ok(d) => {
                        let title = d
                            .find(select::predicate::Name("title"))
                            .next()
                            .unwrap()
                            .children()
                            .next()
                            .unwrap()
                            .text();
                        Some(title.trim().to_string())
                    }
                }
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let _paths = get_paths();
    let _exts = get_valid_exts();

    let mut links = Vec::<Link>::new();
    walk_dirs(&get_paths(), &get_valid_exts(), &mut links)?;
    write_out_file(links);
    Ok(())
}

fn write_out_file(links: Vec<Link>) {
    let mut ch = Channel::default();
    ch.set_title("local channel");
    let mut items = vec![];
    for l in links {
        if l.state == State::Read {
            continue;
        }
        let mut i = Item::default();

        i.set_title(l.fetch_title());
        i.set_link(l.url);
        i.set_pub_date(l.date);
        items.push(i);
    }
    ch.set_items(items);
    ch.pretty_write_to(io::stdout(), b' ', 2).unwrap();
}

fn get_paths<'a>() -> Vec<Box<Path>> {
    vec![
        Box::from(Path::new("/home/rski/Documents")),
        Box::from(Path::new("/home/rski/vimwiki")),
    ]
}

fn get_valid_exts<'a>() -> Vec<&'a str> {
    vec!["org", "vimwiki", "txt", "wiki"]
}

fn interesting_file(p: &Path, _valid_exts: &Vec<&str>) -> bool {
    _valid_exts.iter().any(|&x| match p.extension() {
        None => false,
        Some(ext) => ext.to_str() == Some(x),
    })
}

fn process_entry(_p: &Path, links: &mut Vec<Link>) -> io::Result<()> {
    let mut f = File::open(_p)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    let mut date: Option<DateTime<Utc>> = None;
    let mut state = State::Read;
    for l in buf.lines() {
        if l.starts_with("[[") {
            let l = l.strip_prefix("[[").unwrap().strip_suffix("]]").unwrap();
            let link = produce_link(l, &date, &state);
            links.push(link);
        } else if l.starts_with("http") {
            let link = produce_link(l, &date, &state);
            links.push(link);
        } else if l.starts_with("read") {
            state = State::Read;
        } else if l.starts_with("unread") {
            state = State::Unread;
        } else if !l.is_empty() {
            // Nov 28 12:00:09", "%a %b %e %T %Y"
            let s = format!("{} 09:00:00 2021", l.trim());
            match Utc.datetime_from_str(&s, "%e %B %T %Y") {
                Err(e) => panic!("{}:{}", s, e),
                Ok(v) => date = Some(v),
            }
            state = State::Read; // new day implies read, if not it'll be written down
        }
    }
    Ok(())
}

fn produce_link(s: &str, date: &Option<DateTime<Utc>>, state: &State) -> Link {
    let mut it = s.splitn(2, ' ');
    let url = it.next().unwrap().to_owned();
    Link {
        // some lines have dots but those are not really part of the url
        url: url.strip_suffix('.').unwrap_or(&url).to_owned(),
        tags: vec![],
        date: date
            .clone()
            .unwrap_or(Utc.ymd(2014, 11, 28).and_hms(12, 0, 9))
            .to_rfc2822(),
        state: state.clone(),
    }
}
fn walk_dirs(
    dirs: &Vec<Box<Path>>,
    valid_exts: &Vec<&str>,
    links: &mut Vec<Link>,
) -> io::Result<()> {
    if dirs.is_empty() {
        return Ok(());
    }
    let mut rec: Vec<Box<Path>> = Vec::<Box<Path>>::new();
    for dir in dirs.into_iter() {
        if !Path::is_dir(dir) {
            return Ok(());
        }
        let mut d = dir.read_dir()?;
        for entry in &mut d {
            match entry {
                Err(e) => return Err(e),
                Ok(entry_) => {
                    let p = entry_.path();
                    if Path::is_dir(&p) {
                        rec.push(Box::from(p));
                        continue;
                    }
                    if !interesting_file(&p, &valid_exts) {
                        continue;
                    }
                    process_entry(&p, links)?;
                }
            };
        }
    }
    walk_dirs(&rec, valid_exts, links)?;
    Ok(())
}
