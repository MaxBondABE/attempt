mod arguments;
mod util;

use std::{thread, io};

use arguments::ArgumentParser;
use clap::Parser;

fn main() -> Result<(), io::Error> {
    let args = ArgumentParser::parse();
    let mut command = args.strategy.command();
    let mut success = false;
    for duration in args.strategy {
        if command.status()?.success() {
            success = true;
            break;
        } else {
            thread::sleep(duration);
        }
    }

    std::process::exit(!success as i32);
}
