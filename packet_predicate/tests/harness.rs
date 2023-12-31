use fuels::{
    accounts::{
        wallet::WalletUnlocked,
        predicate::Predicate,
    },
    prelude::*,
    types::{
        input::Input,
        output::Output,
        transaction_builders::ScriptTransactionBuilder,
        transaction_builders::TransactionBuilder,
        Bits256,
        UtxoId,
    },
    tx::{
        Bytes32,
        Receipt,
    },
};

abigen!(Predicate(
    name = "PacketPredicate",
    abi = "./out/debug/packet_predicate-abi.json"
));

pub const TOKEN_A: AssetId = AssetId::new([1u8; 32]);
pub const TOKEN_B: AssetId = AssetId::new([2u8; 32]);
pub const TOKEN_C: AssetId = AssetId::new([7u8; 32]);

pub async fn get_wallets() -> Vec<WalletUnlocked> {
    let num_wallets = 3;

    let asset_ids = [BASE_ASSET_ID, TOKEN_A, TOKEN_B, TOKEN_C];
    let asset_configs = asset_ids
        .map(|id| AssetConfig {
            id,
            num_coins: 1,
            coin_amount: 100_000,
        })
        .into();

    let wallets_config = WalletsConfig::new_multiple_assets(num_wallets, asset_configs);

    let wallets = launch_custom_provider_and_get_wallets(wallets_config, None, None)
        .await
        .unwrap();

    wallets
}

fn get_predicate(signer: Address, provider: &Provider) -> Predicate {
    let configurables = PacketPredicateConfigurables::new().with_SIGNER(signer);

    let mut predicate: Predicate = Predicate::load_from("./out/debug/packet_predicate.bin")
        .unwrap()
        .with_configurables(configurables);
    predicate.set_provider(provider.clone());

    predicate
}

async fn setup() -> (Vec<WalletUnlocked>, Predicate, Provider) {
    let wallets = get_wallets().await;
    let deployer = wallets[0].clone();
    let fuel_provider = deployer.provider().unwrap();

    let mut predicate = get_predicate(deployer.address().into(), &fuel_provider);

    deployer
        .transfer(predicate.address(), 1, TOKEN_A, TxParameters::default())
        .await
        .unwrap();

    (wallets.clone(), predicate, fuel_provider.clone())
}

#[tokio::test]
async fn user_can_include_packet_in_tx() {
    let (wallets, predicate, fuel_provider) = setup().await;
    let user = &wallets[1];

    let mut token_a_inputs = predicate
        .get_asset_inputs_for_amount(TOKEN_A, 1)
        .await
        .unwrap();

    let eth_inputs = user
        .get_asset_inputs_for_amount(BASE_ASSET_ID, 1000)
        .await
        .unwrap();
    token_a_inputs.extend(eth_inputs);

    let outputs = vec![
        Output::Change {
            to: user.address().into(),
            amount: 0,
            asset_id: BASE_ASSET_ID,
        },
        Output::Coin {
            to: predicate.address().into(),
            amount: 1,
            asset_id: TOKEN_A,
        },
    ];

    // Create the Tx
    let mut transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        token_a_inputs,
        outputs,
        TxParameters::default(),
        fuel_provider.network_info().await.unwrap(),
    );

    user.sign_transaction(&mut transaction_builder);

    let script_transaction = transaction_builder.build().unwrap();

    fuel_provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .unwrap();
}

#[tokio::test]
async fn user_cant_take_packet() {
    let (wallets, predicate, fuel_provider) = setup().await;
    let user = &wallets[1];

    let mut token_a_inputs = predicate
        .get_asset_inputs_for_amount(TOKEN_A, 1)
        .await
        .unwrap();

    let eth_inputs = user
        .get_asset_inputs_for_amount(BASE_ASSET_ID, 1000)
        .await
        .unwrap();
    token_a_inputs.extend(eth_inputs);

    let outputs = vec![
        Output::Change {
            to: user.address().into(),
            amount: 0,
            asset_id: BASE_ASSET_ID,
        },
        Output::Coin {
            to: user.address().into(),
            amount: 1,
            asset_id: TOKEN_A,
        },
    ];

    // Create the Tx
    let mut transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        token_a_inputs,
        outputs,
        TxParameters::default(),
        fuel_provider.network_info().await.unwrap(),
    );

    user.sign_transaction(&mut transaction_builder);

    let script_transaction = transaction_builder.build().unwrap();

    let is_err = fuel_provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .is_err();
    assert!(is_err, "User should not be able to take packet");
}

#[tokio::test]
async fn user_cant_burn_packet() {
    let (wallets, predicate, fuel_provider) = setup().await;
    let user = &wallets[1];

    let mut token_a_inputs = predicate
        .get_asset_inputs_for_amount(TOKEN_A, 1)
        .await
        .unwrap();

    let eth_inputs = user
        .get_asset_inputs_for_amount(BASE_ASSET_ID, 1000)
        .await
        .unwrap();
    token_a_inputs.extend(eth_inputs);

    let outputs = vec![
        Output::Change {
            to: user.address().into(),
            amount: 0,
            asset_id: BASE_ASSET_ID,
        },
    ];

    // Create the Tx
    let mut transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        token_a_inputs,
        outputs,
        TxParameters::default(),
        fuel_provider.network_info().await.unwrap(),
    );

    user.sign_transaction(&mut transaction_builder);

    let script_transaction = transaction_builder.build().unwrap();

    let is_err = fuel_provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .is_err();
    assert!(is_err, "User should not be able to take packet");
}
