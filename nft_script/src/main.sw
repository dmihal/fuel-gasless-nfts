script;

use std::constants::ZERO_B256;
use shared::{Mint, PacketMinter};

configurable {
    NFT_CONTRACT: ContractId = ContractId::from(ZERO_B256),
    PACKET_MINTER_CONTRACT: ContractId = ContractId::from(ZERO_B256),
}

fn main(recipient: Address, mint: bool) {
    let nft_contract = abi(Mint, NFT_CONTRACT.into());
    let _out = nft_contract.mint(Identity::Address(recipient));

    if mint {
        let packet_minter_contract = abi(PacketMinter, PACKET_MINTER_CONTRACT.into());
        packet_minter_contract.mint_packet(recipient);
    }
}
