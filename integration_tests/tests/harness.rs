use fuels::{prelude::*, types::ContractId};
use fuels::types::Bits256;
use fuels::tx::Bytes32;
use fuels::types::output::Output;
use fuels::types::TxPointer;
use fuels::types::UtxoId;
use fuels::types::input::Input;
use fuels::types::transaction_builders::ScriptTransactionBuilder;
use fuels::types::transaction_builders::TransactionBuilder;
use fuels::accounts::predicate::Predicate;
use sha2::{Digest, Sha256};
use fuels::tx::Receipt;

abigen!(
    Predicate(name = "GasPredicate", abi = "gas_predicate/out/debug/gas_predicate-abi.json"),
    Script(name = "NFTScript", abi = "nft_script/out/debug/nft_script-abi.json"),
    Contract(name = "NFT", abi = "nft/out/debug/nft-abi.json"),
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
        "../nft/out/debug/nft.bin",
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
    let script = NFTScript::new(account.clone(), "../nft_script/out/debug/nft_script.bin")
        .with_configurables(configurables);

    let mut hasher = Sha256::new();
    hasher.update(script.main(account.address()).script_call.script_binary);
    let b256 = Bits256(hasher.finalize().into());

    (script, b256)
}

fn get_predicate(script_hash: Bits256, signer: Address, nft_contract_id: ContractId, provider: &Provider) -> Predicate {
    let configurables = GasPredicateConfigurables::new()
        .with_SIGNER(signer)
        .with_NFT_CONTRACT_ID(nft_contract_id)
        .with_EXPECTED_SCRIPT_BYTECODE_HASH(script_hash);

    let predicate_data = GasPredicateEncoder::encode_data(vec![]);

    let mut predicate: Predicate = Predicate::load_from("../gas_predicate/out/debug/gas_predicate.bin")
        .unwrap()
        .with_data(predicate_data)
        .with_configurables(configurables);
    predicate.set_provider(provider.clone());

    predicate
}

#[tokio::test]
async fn can_use_script() {
    let wallets = get_wallets().await;
    let deployer = &wallets[0];
    let user = &wallets[1];
    let user_2 = &wallets[2];
    let fuel_provider = deployer.provider().unwrap();
    let network_info = fuel_provider.network_info().await.unwrap();

    let nft_instance = get_contract_instance(deployer).await;
    let (_script, script_hash) = get_script(user.clone(), nft_instance.id().into()).await;
    let predicate = get_predicate(
        script_hash,
        deployer.address().into(),
        nft_instance.id().into(),
        deployer.provider().unwrap(),
    );

    deployer.transfer(predicate.address(), 10000, BASE_ASSET_ID, TxParameters::default())
        .await
        .unwrap();

    let (script, _script_hash) = get_script(predicate.clone(), nft_instance.id().into()).await;

    let mut inputs = vec![
        Input::Contract {
            utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
            tx_pointer: TxPointer::default(),
            contract_id: nft_instance.id().into(),
        },
    ];

    let eth_inputs = predicate
        .get_asset_inputs_for_amount(BASE_ASSET_ID, 1000)
        .await
        .unwrap();
    inputs.extend(eth_inputs);

    let outputs = vec![
        Output::Contract {
            input_index: 0u8,
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
        },
        Output::Variable {
            to: Address::default(),
            amount: 0,
            asset_id: AssetId::default(),
        },
        Output::Change {
            to: predicate.address().into(),
            amount: 0,
            asset_id: BASE_ASSET_ID,
        }
    ];

    // Create the Tx
    let transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        inputs,
        outputs,
        TxParameters::default(),
        network_info.clone(),
    )
        .with_script(script.main(user.address()).script_call.script_binary)
        .with_script_data(script.main(user.address()).script_call.encoded_args.resolve(0));

    let mut script_transaction = transaction_builder.build().unwrap();

    let expected_tx_id = script_transaction.id(network_info.chain_id());
    let signature = deployer.sign_message(expected_tx_id).await.unwrap();
    script_transaction.append_witness(signature.as_ref().into());

    let actual_tx_id = fuel_provider.send_transaction_and_await_commit(script_transaction).await.unwrap();
    assert_eq!(expected_tx_id, actual_tx_id);

    let tx_status = fuel_provider.tx_status(&actual_tx_id).await.unwrap();
    let receipts = tx_status.take_receipts_checked(None).unwrap();

    let mut nft_asset_id: Option<AssetId> = None;
    let mut nft_sub_id: Option<Bytes32> = None;
    for item in receipts.iter() {
        match item {
            Receipt::Mint{ contract_id, sub_id, .. } => {
                let nft_id: ContractId = nft_instance.id().into();
                assert!(contract_id.clone() == nft_id);
                nft_sub_id = Some(*sub_id);
            },
            Receipt::TransferOut{ to, asset_id, .. } => {
                let user_address: Address = user.address().into();
                assert!(to.clone() == user_address);
                nft_asset_id = Some(*asset_id);
            },
            Receipt::LogData{ data, .. } => {
                println!("LogData: {}", vec_to_str(&data.clone().unwrap()));
            },
            _ => {},
        }
    }

    // =================
    // Do basic transfer
    // =================

    let predicate = predicate.with_data(GasPredicateEncoder::encode_data(vec![Bits256(nft_sub_id.unwrap().into())]));

    let mut nft_inputs = user
        .get_asset_inputs_for_amount(nft_asset_id.unwrap(), 1)
        .await
        .unwrap();

    let eth_inputs = predicate
        .get_asset_inputs_for_amount(BASE_ASSET_ID, 1000)
        .await
        .unwrap();
    nft_inputs.extend(eth_inputs);

    let outputs = vec![
        Output::Change {
            to: predicate.address().into(),
            amount: 0,
            asset_id: BASE_ASSET_ID,
        },
        Output::Coin {
            to: user_2.address().into(),
            amount: 1,
            asset_id: nft_asset_id.unwrap(),
        },
    ];

    // Create the Tx
    let mut transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        nft_inputs,
        outputs,
        TxParameters::default(),
        network_info.clone(),
    );

    user.sign_transaction(&mut transaction_builder);

    let mut script_transaction = transaction_builder.build().unwrap();

    let expected_tx_id = script_transaction.id(network_info.chain_id());
    let signature = deployer.sign_message(expected_tx_id).await.unwrap();
    script_transaction.append_witness(signature.as_ref().into());

    let actual_tx_id = fuel_provider.send_transaction_and_await_commit(script_transaction).await.unwrap();
    assert_eq!(expected_tx_id, actual_tx_id);

    let tx_status = fuel_provider.tx_status(&actual_tx_id).await.unwrap();
    let _receipts = tx_status.take_receipts_checked(None).unwrap();
}

fn vec_to_str(vec: &Vec<u8>) -> String {
    vec.iter().map(|b| format!("{:02x}", b)).collect()
}