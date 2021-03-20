#![no_main]
use libfuzzer_sys::fuzz_target;

// Fuzz the deserialize() function with random bytes to check that it doesn't crash on invalid
// inputs.
fuzz_target!(|data: &[u8]| {
    gpg_keytag::keyfile::deserialize(data);
});
