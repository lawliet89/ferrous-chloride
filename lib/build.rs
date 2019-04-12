extern crate version_check;
extern crate yansi;

use version_check::{is_min_date, is_min_version, supports_features};
use yansi::Color::{Blue, Red, Yellow};

// Specifies the minimum nightly version needed to compile
const MIN_DATE: &'static str = "2019-04-10";
const MIN_VERSION: &'static str = "1.34.0-stable";

fn main() {
    let ok_channel = supports_features();
    let ok_version = is_min_version(MIN_VERSION);
    let ok_date = is_min_date(MIN_DATE);
    let triple = (ok_channel, ok_version, ok_date);

    let print_version_err = |version: &str, date: &str| {
        eprintln!(
            "{} {}. {} {}.",
            "Installed version is:",
            Yellow.paint(format!("{} ({})", version, date)),
            "Minimum required:",
            Yellow.paint(format!("{} ({})", MIN_VERSION, MIN_DATE))
        );
    };

    if let (Some(_ok_channel), Some((ok_version, version)), Some((ok_date, date))) = triple {
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
            print_version_err(&*version, &*date);
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
