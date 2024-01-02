use fuels::{
    accounts::{
        predicate::Predicate,
        wallet::{Wallet, WalletUnlocked},
    },
    prelude::Error::RevertTransactionError,
    prelude::*,
    types::Bits256,
};
use sha2::{Digest, Sha256};

abigen!(
    Predicate(
        name = "GasPredicate",
        abi = "gas_predicate/out/debug/gas_predicate-abi.json"
    ),
    Script(
        name = "NFTScript",
        abi = "nft_script/out/debug/nft_script-abi.json"
    ),
    Contract(name = "NFT", abi = "nft/out/debug/nft-abi.json"),
    Contract(
        name = "PacketMinter",
        abi = "packet_minter/out/debug/packet_minter-abi.json"
    ),
);

pub async fn get_wallets() -> Vec<WalletUnlocked> {
    // Launch a local network and deploy the contract
    let wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(3),             /* Single wallet */
            Some(2),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();

    wallets
}

pub async fn get_nft_contract_instance(wallet: &WalletUnlocked) -> NFT<WalletUnlocked> {
    let configurables = NFTConfigurables::new().with_MAX_SUPPLY(1000);
    let id = Contract::load_from(
        "../nft/out/debug/nft.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )
    .unwrap()
    .deploy(wallet, TxParameters::default())
    .await
    .unwrap();

    NFT::new(id, wallet.clone())
}

pub async fn get_packet_minter_contract_instance(
    wallet: &WalletUnlocked,
) -> PacketMinter<WalletUnlocked> {
    let id = Contract::load_from(
        "../packet_minter/out/debug/packet_minter.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(wallet, TxParameters::default())
    .await
    .unwrap();

    PacketMinter::new(id, wallet.clone())
}

pub async fn get_script<T: Account>(
    account: T,
    nft: ContractId,
    packet_minter: ContractId,
) -> (NFTScript<T>, Bits256) {
    let configurables = NFTScriptConfigurables::new()
        .with_NFT_CONTRACT(nft)
        .with_PACKET_MINTER_CONTRACT(packet_minter);
    let script = NFTScript::new(account.clone(), "../nft_script/out/debug/nft_script.bin")
        .with_configurables(configurables);

    let mut hasher = Sha256::new();
    hasher.update(
        script
            .main(account.address(), false)
            .script_call
            .script_binary,
    );
    let b256 = Bits256(hasher.finalize().into());

    (script, b256)
}

pub fn get_predicate(
    script_hash: Bits256,
    signer: Address,
    nft_contract_id: ContractId,
    provider: &Provider,
) -> Predicate {
    let configurables = GasPredicateConfigurables::new()
        .with_SIGNER(signer)
        .with_NFT_CONTRACT_ID(nft_contract_id)
        .with_EXPECTED_SCRIPT_BYTECODE_HASH(script_hash);

    let predicate_data = GasPredicateEncoder::encode_data(vec![]);

    let mut predicate: Predicate =
        Predicate::load_from("../gas_predicate/out/debug/gas_predicate.bin")
            .unwrap()
            .with_data(predicate_data)
            .with_configurables(configurables);
    predicate.set_provider(provider.clone());

    predicate
}
