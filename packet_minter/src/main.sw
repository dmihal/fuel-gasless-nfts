contract;

use shared::PacketMinter;
use std::{
    b512::B512,
    constants::ZERO_B256,
    token::mint_to_address,
    tx::{tx_id, tx_witness_data, tx_witnesses_count},
    ecr::ec_recover_address,
    hash::{sha256, Hash},
};

enum Errors {
    InvalidSignature: (),
}

storage {
    signer: Address = Address::from(ZERO_B256),
    packet_predicate: Address = Address::from(ZERO_B256),
}

impl PacketMinter for Contract {
    #[storage(read)]
    fn mint_packet(subject: Address) {
        ensure_tx_signed();

        mint_to_address(storage.packet_predicate.read(), subject.into(), 1);
    }
}

#[storage(read)]
fn ensure_tx_signed() {
    let signature: B512 = tx_witness_data(tx_witnesses_count() - 1);

    let expected_signer = storage.signer.read();

    let signer_address = ec_recover_address(signature, sha256(tx_id()));
    require(signer_address.is_ok() && signer_address.unwrap() == expected_signer, Errors::InvalidSignature);
}

abi PacketMinterAdmin {
    #[storage(read, write)]
    fn set_signer(signer: Address);
    #[storage(read, write)]
    fn set_packet_predicate(packet_addr: Address);
}

impl PacketMinterAdmin for Contract {
    #[storage(read, write)]
    fn set_signer(signer: Address) {
        storage.signer.write(signer);
    }

    #[storage(read, write)]
    fn set_packet_predicate(packet_addr: Address) {
        storage.packet_predicate.write(packet_addr);
    }
}
