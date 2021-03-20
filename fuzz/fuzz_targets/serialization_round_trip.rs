#![no_main]
use libfuzzer_sys::fuzz_target;

use gpg_keytag::keyfile::{deserialize, serialize, TokenTree};

// Serialize-deserialize random token trees and check that we get the initial tree back.
fuzz_target!(|tree: TokenTree| {
    let mut buffer: Vec<u8> = Vec::new();
    serialize(&tree, &mut buffer).unwrap();
    assert_eq!(deserialize(&buffer).unwrap(), tree);
});
