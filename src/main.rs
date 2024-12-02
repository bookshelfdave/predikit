// Copyright (c) 2025 Dave Parfitt

use ::predikit::process_cli;

// this is pub to get rid of compile warnings
pub mod predikit;

fn main() {
    if !process_cli() {
        // Guessing that there's a better way to do this in 2025
        std::process::exit(1);
    }
}
