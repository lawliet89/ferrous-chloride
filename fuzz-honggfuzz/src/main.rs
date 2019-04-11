#[macro_use]
extern crate honggfuzz;

use ferrous_chloride::parse_str;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            if let Ok(s) = std::str::from_utf8(data) {
                let _ = parse_str(s);
            }
        });
    }
}
