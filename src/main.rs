mod arguments;
mod util;

use std::{thread, io};

use arguments::ArgumentParser;
use clap::Parser;

fn main() -> Result<(), io::Error> {
    let args = ArgumentParser::parse();
    let mut command = args.backoff.command();
    for duration in args.backoff {
        if command.status()?.success() {
            std::process::exit(0);
        } else {
            thread::sleep(duration);
        }
    }

    std::process::exit(1);
}
