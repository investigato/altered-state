//! Launch and end banners
use crate::utilities::date::{return_current_date, return_current_time};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

/// Start banner
pub fn print_banner() {
    // https://docs.rs/colored/2.0.0/x86_64-pc-windows-msvc/colored/control/fn.set_virtual_terminal.html
    #[cfg(windows)]
    control::set_virtual_terminal(true).unwrap();

    println!(
        "{}",
        "---------------------------------------------------"
            .clear()
            .bold()
    );
    println!(
        "Initializing {} at {} on {}",
        "Retcon".truecolor(0, 176, 0,),
        return_current_time(),
        return_current_date()
    );
    println!("{}", "investigato".bold());
    println!(
        "{}\n",
        "---------------------------------------------------"
            .clear()
            .bold()
    );
}

/// Banner at end.
pub fn print_end_banner() {
    println!("\nWe done at {}!", return_current_time(),);
}

/// Progress Bar.
pub fn progress_bar(pb: ProgressBar, message: String, count: u64, end_message: String) {
    pb.set_style(
        ProgressStyle::with_template("{prefix:.bold.dim}{spinner} {wide_msg}")
            .unwrap()
            .tick_chars("------"),
    );
    pb.inc(count);
    pb.with_message(format!("{}: {}{}", message, count, end_message));
}
