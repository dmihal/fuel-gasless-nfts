use fuels::{prelude::*, types::ContractId};
use fuels::types::Bits256;
use fuels::accounts::predicate::Predicate;
use sha2::{Digest, Sha256};
use fuels::tx::Receipt;

abigen!(
    Predicate(name = "GasPredicate", abi = "./gas_predicate/out/debug/gas_predicate-abi.json"),
    Script(name = "NFTScript", abi = "./nft_script/out/debug/nft_script-abi.json"),
    Contract(name = "NFT", abi = "./nft/out/debug/nft-abi.json"),
);

async fn get_wallets() -> Vec<WalletUnlocked> {
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

async fn get_contract_instance(wallet: &WalletUnlocked) -> NFT<WalletUnlocked> {
    let configurables = NFTConfigurables::new().with_MAX_SUPPLY(1000);
    let id = Contract::load_from(
        "./nft/out/debug/nft.bin",
        LoadConfiguration::default().with_configurables(configurables),
    )
    .unwrap()
    .deploy(wallet, TxParameters::default())
    .await
    .unwrap();

    let instance = NFT::new(id, wallet.clone());

    instance
}

async fn get_script<T: Account>(account: T, nft: ContractId) -> (NFTScript<T>, Bits256) {
    let configurables = NFTScriptConfigurables::new().with_NFT_CONTRACT(nft);
    let script = NFTScript::new(account.clone(), "./nft_script/out/debug/nft_script.bin")
        .with_configurables(configurables);

    let mut hasher = Sha256::new();
    hasher.update(script.main(account.address()).script_call.script_binary);
    let b256 = Bits256(hasher.finalize().into());

    (script, b256)
}

fn get_predicate(script_hash: Bits256, provider: &Provider) -> Predicate {
    let configurables = GasPredicateConfigurables::new()
        .with_EXPECTED_SCRIPT_BYTECODE_HASH(script_hash);

    let mut predicate: Predicate = Predicate::load_from("./gas_predicate/out/debug/gas_predicate.bin")
        .unwrap()
        .with_configurables(configurables);
    predicate.set_provider(provider.clone());

    predicate
}

#[tokio::test]
async fn can_use_script() {
    let wallets = get_wallets().await;
    let deployer = &wallets[0];
    let user = &wallets[1];

    let nft_instance = get_contract_instance(deployer).await;
    let (_script, script_hash) = get_script(user.clone(), nft_instance.id().into()).await;
    let predicate = get_predicate(script_hash, deployer.provider().unwrap());

    deployer.transfer(predicate.address(), 10000, BASE_ASSET_ID, TxParameters::default())
        .await
        .unwrap();

    let (script, _script_hash) = get_script(predicate, nft_instance.id().into()).await;

    let result = script
        .main(user.address())
        .with_contracts(&[&nft_instance])
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    println!("{:?}", result);

    for item in result.receipts.iter() {
        match item {
            Receipt::Mint{ sub_id, contract_id, .. } => {
                let nft_id: ContractId = nft_instance.id().into();
                assert!(contract_id.clone() == nft_id);
            },
            Receipt::TransferOut{ to, .. } => {
                let user_address: Address = user.address().into();
                assert!(to.clone() == user_address);
            },
            _ => {},
        }
    }
}

// #[tokio::test]
// async fn cant_do_transfer() {
//     let wallets = get_wallets().await;
//     let deployer = &wallets[0];
//     let user = &wallets[1];

//     let contract_instance = get_contract_instance(deployer).await;
//     let (script, script_hash) = get_script(user.clone(), contract_instance.id().into()).await;
//     let predicate = get_predicate(script_hash, deployer.provider().unwrap());

//     deployer.transfer(predicate.address(), 10000, BASE_ASSET_ID, TxParameters::default())
//         .await
//         .unwrap();

//     let is_err = predicate.transfer(user.address(), 10000, BASE_ASSET_ID, TxParameters::default())
//         .await
//         .is_err();
//     assert!(is_err);
// }
