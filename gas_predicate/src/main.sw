predicate;

use std::{
    constants::ZERO_B256,
    b512::B512,
    hash::{sha256, Hash},
    inputs::{
        Input,
        input_asset_id,
        input_count,
        input_owner,
        input_type,
    },
    tx::{
        tx_id,
        tx_witness_data,
        tx_script_bytecode_hash,
        tx_script_length,
        tx_witnesses_count,
    },
    ecr::ec_recover_address,
};

configurable {
    SIGNER: Address = Address::from(ZERO_B256),
    NFT_CONTRACT_ID: ContractId = ContractId::from(ZERO_B256),
    EXPECTED_SCRIPT_BYTECODE_HASH: b256 = ZERO_B256,
}

const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;


fn main(sub_ids: Vec<SubId>) -> bool {
    let signature: B512 = tx_witness_data(tx_witnesses_count() - 1);

    let signer_address = ec_recover_address(signature, sha256(tx_id())).unwrap();
    if (signer_address != SIGNER) {
        return false;
    }

    let is_script_valid = if (tx_script_length() > 0) {
        let script_bytecode_hash: b256 = tx_script_bytecode_hash();
        script_bytecode_hash == EXPECTED_SCRIPT_BYTECODE_HASH
    } else { true };
    if (!is_script_valid) {
        return false;
    }

    let predicate_addr = predicate_address();
    if (NFT_CONTRACT_ID == ContractId::from(ZERO_B256)) {
        return false;
    }

    let mut i = 0;

    let mut nft_asset_ids: Vec<AssetId> = Vec::with_capacity(sub_ids.len);
    while i < sub_ids.len {
        nft_asset_ids.push(AssetId::new(NFT_CONTRACT_ID, sub_ids.get(i).unwrap()));
        i = i + 1;
    }

    let num_inputs = input_count().as_u64();
    i = 0;
    while i < num_inputs {
        match input_type(i) {
            Input::Coin => {
                let asset_id = input_asset_id(i).unwrap();
                if (asset_id == AssetId::from(ZERO_B256)) {
                    let owner = input_owner(i).unwrap();
                    if (owner != predicate_addr) {
                        return false;
                    }
                } else {
                    if !asset_exists_in_vec(asset_id, nft_asset_ids) {
                        return false;
                    }
                }
            },
            Input::Message => {
                revert(0);
            },
            Input::Contract => {
                let contract_id = input_contract_id(i).unwrap();
                if (contract_id != NFT_CONTRACT_ID) {
                    return false;
                }
            },
        }
        i = i + 1;
    }

    return true;
}

fn predicate_address() -> Address {
    let predicate_index = asm(r1) {
        gm r1 i3;
        r1: u64
    };
    input_owner(predicate_index).unwrap()
}


fn input_contract_id(index: u64) -> Option<ContractId> {
    match input_type(index) {
        Input::Contract => {
            let addr_ptr = __gtf::<raw_ptr>(index, GTF_INPUT_CONTRACT_CONTRACT_ID);
            // Why do I have to add 2?
            Some(addr_ptr.add::<u64>(2).read::<ContractId>())
        },
        _ => None,
    }
}

fn asset_exists_in_vec(asset_id: AssetId, nft_asset_ids: Vec<AssetId>) -> bool {
    let mut i = 0;
    while i < nft_asset_ids.len {
        if asset_id == nft_asset_ids.get(i).unwrap() {
            return true;
        }
        i = i + 1;
    }
    false
}
