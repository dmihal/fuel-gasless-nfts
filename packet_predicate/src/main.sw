predicate;

use std::{
    b512::B512,
    constants::ZERO_B256,
    ecr::ec_recover_address,
    hash::{
        Hash,
        sha256,
    },
    inputs::{
        input_amount,
        input_asset_id,
        input_owner,
    },
    outputs::{
        Output,
        output_amount,
        output_asset_id,
        output_asset_to,
        output_count,
        output_type,
    },
    tx::{
        tx_id,
        tx_witness_data,
    },
};

configurable {
    SIGNER: Address = Address::from(ZERO_B256),
}

fn main(signature_witness_id: Option<u64>) -> bool {
    match signature_witness_id {
        Some(signature_witness_id) => {
            let signature: B512 = tx_witness_data(signature_witness_id);
            let signer_address = ec_recover_address(signature, sha256(tx_id())).unwrap();
            signer_address == SIGNER
        }
        None => {
            let input_index = predicate_input_index();
            let predicate_address = input_owner(input_index).unwrap();
            let input_id = input_asset_id(input_index).unwrap();
            let input_amt = input_amount(input_index).unwrap();

            let num_outputs = output_count();
            let mut i = 0;
            let mut returns_eth_to_predicate = false;

            while i < num_outputs {
                match output_type(i) {
                    Output::Coin => {
                        if output_asset_id(i).unwrap() == input_id
                            && output_amount(i) == input_amt
                            && Address::from(output_asset_to(i).unwrap()) == predicate_address
                        {
                            return true;
                        }
                    },
                    _ => (),
                }
                i = i + 1;
            }

            false
        },
    }
}

fn predicate_input_index() -> u64 {
    asm(r1) {
        gm   r1 i3;
        r1: u64
    }
}
