use std::borrow::Cow;

use version_check::{is_min_date, is_min_version, triple};
use yansi::Color::{Blue, Red, Yellow};

// Specifies the minimum nightly version needed to compile
const MIN_DATE: &'static str = "2019-04-10";
const MIN_VERSION: &'static str = "1.34.0-stable";

fn print_version_err() {
    let (version, date) = match triple() {
        Some((version, _channel, date)) => (
            Cow::Owned(version.to_string()),
            Cow::Owned(date.to_string()),
        ),
        None => (Cow::Borrowed("Unknown"), Cow::Borrowed("Unknown")),
    };

    eprintln!(
        "{} {}. {} {}.",
        "Installed version is:",
        Yellow.paint(format!("{} ({})", version, date)),
        "Minimum required:",
        Yellow.paint(format!("{} ({})", MIN_VERSION, MIN_DATE))
    );
}

fn main() {
    let ok_version = is_min_version(MIN_VERSION);
    let ok_date = is_min_date(MIN_DATE);
    let double = (ok_version, ok_date);

    if let (Some(ok_version), Some(ok_date)) = double {
        if !ok_version || !ok_date {
            eprintln!(
                "{} {}",
                Red.paint("Error:").bold(),
                format!(
                    "A more recent version of Rust is needed. ({} {})",
                    MIN_VERSION, MIN_DATE
                )
            );
            eprintln!(
                "{}{}{}",
                Blue.paint("Use `"),
                "rustup update",
                Blue.paint("` or your preferred method to update Rust.")
            );
            print_version_err();
            panic!("Aborting compilation due to incompatible compiler.")
        }
    } else {
        println!(
            "cargo:warning={}",
            "Ferrous Chloride was unable to check rustc compatibility."
        );
        println!(
            "cargo:warning={}",
            "Build may fail due to incompatible rustc version."
        );
    }
}
