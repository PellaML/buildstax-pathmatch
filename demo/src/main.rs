use buildstax_pathmatch::Ignore;
use std::{
    error::Error,
    io::{self, BufRead},
};

fn main() -> Result<(), Box<dyn Error>> {
    let rules = std::env::args_os()
        .nth(1)
        .ok_or("usage: pathmatch-demo RULEFILE (paths on stdin; trailing / marks directories)")?;
    let engine = Ignore::new(&std::fs::read_to_string(rules)?)?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        let (path, dir) = line
            .strip_suffix('/')
            .map_or((line.as_str(), false), |p| (p, true));
        println!("{line}: {:?}", engine.decide(path, dir));
    }
    Ok(())
}
