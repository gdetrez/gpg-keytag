#!/bin/sh

cd $SRC/gpg-keytag
cargo fuzz build -O
cp fuzz/target/x86_64-unknown-linux-gnu/release/{deerialization,serialization_round_trip} $OUT/
