#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    field: u8,
    text: String,
}

fuzz_target!(|input: Input| {
});
