use std::{io, path::Path};

use rss::{Channel, Item};

fn main() {
    let _paths = get_paths();
    let _exts = get_valid_exts();
    let mut ar = Item::default();
    ar.set_author(String::from("foo"));

    let mut ch = Channel::default();
    ch.set_items(vec![ar]);
    ch.pretty_write_to(io::stdout(), b' ', 2).unwrap();

    walk_dirs(&get_paths(), &get_valid_exts()).unwrap();
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

fn process_entry(p: &Path, _valid_exts: &Vec<&str>) {
    if !_valid_exts.iter().any(|&x| match p.extension() {
        None => false,
        Some(ext) => ext.to_str() == Some(x),
    }) {
        return;
    }
    println!("{:?}", p.to_str());
}

fn walk_dirs(dirs: &Vec<Box<Path>>, valid_exts: &Vec<&str>) -> io::Result<()> {
    if dirs.is_empty() {
        return Ok(());
    }
    let mut rec: Vec<Box<Path>> = Vec::<Box<Path>>::new();
    for dir in dirs {
        if !Path::is_dir(dir) {
            continue;
        }
        let mut d = dir.read_dir()?;
        for entry in &mut d {
            match entry {
                Err(e) => return Err(e),
                Ok(entry_) => {
                    let p = entry_.path();
                    if Path::is_dir(&p) {
                        rec.push(Box::from(p));
                    } else {
                        process_entry(&p, &valid_exts);
                    }
                }
            };
        }
    }
    walk_dirs(&rec, valid_exts)?;

    Ok(())
}
