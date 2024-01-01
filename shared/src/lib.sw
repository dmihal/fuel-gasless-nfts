library;

abi Mint {
    #[storage(read, write)]
    fn mint(recipient: Identity);
}

abi PacketMinter {
    #[storage(read)]
    fn mint_packet(recipient: Address);
}
