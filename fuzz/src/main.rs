#[macro_use]
extern crate honggfuzz;

use ferrous_chloride::parser::parse_str;

fn main() {
    loop {
        fuzz!(|data: &[u8]| {
            let string = std::str::from_utf8(data);
            if let Ok(valid_string) = string {
                let _ = parse_str(valid_string);
            }
        });
    }
}
