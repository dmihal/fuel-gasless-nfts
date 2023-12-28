predicate;

use std::{
    constants::ZERO_B256,
    tx::tx_script_bytecode_hash,
    b512::B512,
    hash::{sha256, Hash},
    tx::{
        tx_id,
        tx_witness_data,
    },
    ecr::ec_recover_address,
};

configurable {
    SIGNER: Address = Address::from(ZERO_B256),
    EXPECTED_SCRIPT_BYTECODE_HASH: b256 = ZERO_B256,
}

fn main() -> bool {
    let script_bytecode_hash: b256 = tx_script_bytecode_hash();
    if (script_bytecode_hash != EXPECTED_SCRIPT_BYTECODE_HASH) {
        return false;
    }

    let signature: B512 = tx_witness_data(0);

    let signer_address = ec_recover_address(signature, sha256(tx_id())).unwrap();
    if (signer_address != SIGNER) {
        return false;
    }

    return true;
}
