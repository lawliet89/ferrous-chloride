#[cfg(not(target_os = "windows"))]
mod non_windows {
    use afl::fuzz;
    use ferrous_chloride::parse_str;

    #[rustfmt::skip]
    pub fn main() {
        loop {
            fuzz!(|data: &[u8]| {
                if let Ok(s) = std::str::from_utf8(data) {
                    let _ = parse_str(&s);
                }
            });
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    pub fn main() {
        panic!(
            "afl does not build in Windows but \
             works well under WSL (Windows Subsystem for Linux)"
        );
    }
}

#[cfg(not(target_os = "windows"))]
use non_windows as implementation;

#[cfg(target_os = "windows")]
use windows as implementation;

fn main() {
    implementation::main();
}
