predicate;

use std::{
    constants::ZERO_B256,
    tx::tx_script_bytecode_hash,
};

configurable {
    EXPECTED_SCRIPT_BYTECODE_HASH: b256 = ZERO_B256,
}

fn main() -> bool {
    let script_bytecode_hash: b256 = tx_script_bytecode_hash();
    return script_bytecode_hash == EXPECTED_SCRIPT_BYTECODE_HASH;
}
