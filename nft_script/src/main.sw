script;

use std::constants::ZERO_B256;
use shared::Mint;

configurable {
    NFT_CONTRACT: ContractId = ContractId::from(ZERO_B256),
}

fn main(recipient: Address) {
    let nft_contract = abi(Mint, NFT_CONTRACT.into());
    let _out = nft_contract.mint(Identity::Address(recipient));
}
