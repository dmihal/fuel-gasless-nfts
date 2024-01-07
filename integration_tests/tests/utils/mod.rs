use fuels::{
    accounts::{predicate::Predicate, wallet::WalletUnlocked},
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
    Predicate(
        name = "PacketPredicate",
        abi = "packet_predicate/out/debug/packet_predicate-abi.json"
    )
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
    packet_predicate: Address,
) -> PacketMinter<WalletUnlocked> {
    let id = Contract::load_from(
        "../packet_minter/out/debug/packet_minter.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(wallet, TxParameters::default())
    .await
    .unwrap();

    let contract = PacketMinter::new(id, wallet.clone());

    contract
        .methods()
        .set_signer(wallet.address())
        .call()
        .await
        .unwrap();
    contract
        .methods()
        .set_packet_predicate(packet_predicate)
        .call()
        .await
        .unwrap();

    contract
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

pub async fn get_gas_predicate<T: Account>(
    script_hash: Bits256,
    signer: &T,
    nft_contract_id: ContractId,
    packet_minter_contract_id: ContractId,
    provider: &Provider,
) -> Predicate {
    let configurables = GasPredicateConfigurables::new()
        .with_SIGNER(signer.address().into())
        .with_NFT_CONTRACT_ID(nft_contract_id)
        .with_PACKET_MINTER_CONTRACT_ID(packet_minter_contract_id)
        .with_EXPECTED_SCRIPT_BYTECODE_HASH(script_hash);

    let predicate_data = GasPredicateEncoder::encode_data(vec![], None);

    let mut predicate: Predicate =
        Predicate::load_from("../gas_predicate/out/debug/gas_predicate.bin")
            .unwrap()
            .with_data(predicate_data)
            .with_configurables(configurables);
    predicate.set_provider(provider.clone());

    signer
        .transfer(
            predicate.address(),
            10000,
            BASE_ASSET_ID,
            TxParameters::default(),
        )
        .await
        .unwrap();

    predicate
}

fn get_packet_predicate(signer: Address, provider: &Provider) -> Predicate {
    let configurables = PacketPredicateConfigurables::new().with_SIGNER(signer);

    let mut predicate: Predicate =
        Predicate::load_from("../packet_predicate/out/debug/packet_predicate.bin")
            .unwrap()
            .with_configurables(configurables);
    predicate.set_provider(provider.clone());

    predicate
}

pub struct Fixture<T: Account> {
    pub wallets: Vec<WalletUnlocked>,
    pub deployer: WalletUnlocked,
    pub user: WalletUnlocked,
    pub nft_instance: NFT<WalletUnlocked>,
    pub packet_minter_instance: PacketMinter<WalletUnlocked>,
    pub script: NFTScript<T>,
    pub gas_predicate: Predicate,
    pub packet_predicate: Predicate,
}

pub async fn setup() -> Fixture<Predicate> {
    let wallets = get_wallets().await;
    let deployer = &wallets[0];
    let user = &wallets[1];

    let nft_instance = get_nft_contract_instance(deployer).await;

    let packet_predicate =
        get_packet_predicate(deployer.address().into(), deployer.provider().unwrap());

    let packet_minter_instance =
        get_packet_minter_contract_instance(deployer, packet_predicate.address().into()).await;

    let (_script, script_hash) = get_script(
        user.clone(),
        nft_instance.id().into(),
        packet_minter_instance.id().into(),
    )
    .await;

    let gas_predicate = get_gas_predicate(
        script_hash,
        deployer,
        nft_instance.id().into(),
        packet_minter_instance.id().into(),
        deployer.provider().unwrap(),
    )
    .await;

    let (script, _script_hash) = get_script(
        gas_predicate.clone(),
        nft_instance.id().into(),
        packet_minter_instance.id().into(),
    )
    .await;

    Fixture {
        deployer: deployer.clone(),
        user: user.clone(),
        wallets,
        nft_instance,
        packet_minter_instance,
        script,
        gas_predicate,
        packet_predicate,
    }
}
