//! Launch and end banners
use chrono::Local;
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

/// Function to return current hours.
pub fn return_current_time() -> String {
    Local::now().format("%T").to_string()
}

/// Function to return current date.
pub fn return_current_date() -> String {
    Local::now().format("%D").to_string()
}
