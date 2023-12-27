library;

abi Mint {
    #[storage(read, write)]
    fn mint(recipient: Identity);
}
