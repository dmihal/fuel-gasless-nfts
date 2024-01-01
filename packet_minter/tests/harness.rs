use fuels::{
    accounts::wallet::{Wallet, WalletUnlocked},
    prelude::Error::RevertTransactionError,
    prelude::*,
    types::Bits256,
};

abigen!(Contract(
    name = "PacketMinter",
    abi = "packet_minter/out/debug/packet_minter-abi.json"
));

const PACKET_ADDRESS: Address = Address::new([1u8; 32]);

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

async fn get_contract_instance(wallet: &WalletUnlocked) -> PacketMinter<WalletUnlocked> {
    let id = Contract::load_from(
        "./out/debug/packet_minter.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(wallet, TxParameters::default())
    .await
    .unwrap();

    PacketMinter::new(id, wallet.clone())
}

async fn setup() -> (Vec<WalletUnlocked>, PacketMinter<WalletUnlocked>, Provider) {
    let wallets = get_wallets().await;
    let deployer = wallets[0].clone();
    let fuel_provider = deployer.provider().unwrap();

    let contract = get_contract_instance(&deployer).await;

    contract
        .methods()
        .set_signer(deployer.address())
        .call()
        .await
        .unwrap();
    contract
        .methods()
        .set_packet_predicate(PACKET_ADDRESS)
        .call()
        .await
        .unwrap();

    (wallets.clone(), contract, fuel_provider.clone())
}

#[tokio::test]
async fn will_mint_packet_with_valid_signature() {
    let (wallets, contract, fuel_provider) = setup().await;
    let deployer = &wallets[0];
    let user = &wallets[1];

    let handler = contract
        .methods()
        .mint_packet(user.address())
        .append_variable_outputs(1);
    let mut tx = handler.build_tx().await.unwrap();

    let network_info = fuel_provider.network_info().await.unwrap();
    let expected_tx_id = tx.id(network_info.chain_id());
    let signature = deployer.sign_message(expected_tx_id).await.unwrap();
    tx.append_witness(signature.as_ref().into());

    let actual_tx_id = fuel_provider
        .send_transaction_and_await_commit(tx)
        .await
        .unwrap();
    let status = fuel_provider.tx_status(&actual_tx_id).await.unwrap();
    status.check(Some(&handler.log_decoder)).unwrap();

    let expected_asset_id = contract
        .id()
        .asset_id(&Bits256(user.address().hash().into()));

    let packet_account = Wallet::from_address(PACKET_ADDRESS.into(), Some(fuel_provider.clone()));
    let packets = packet_account.get_coins(expected_asset_id).await.unwrap();
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0].amount, 1);
}

#[tokio::test]
async fn wont_mint_packet_with_invalid_signature() {
    let (wallets, contract, fuel_provider) = setup().await;
    let user = &wallets[1];

    let handler = contract
        .methods()
        .mint_packet(user.address())
        .append_variable_outputs(1);
    let mut tx = handler.build_tx().await.unwrap();
    tx.append_witness([0u8; 64][..].into());

    let actual_tx_id = fuel_provider
        .send_transaction_and_await_commit(tx)
        .await
        .unwrap();
    let status = fuel_provider.tx_status(&actual_tx_id).await.unwrap();

    match status.take_receipts_checked(Some(&handler.log_decoder)) {
        Ok(_) => panic!("Expected error"),
        Err(err) => {
            if let RevertTransactionError { reason, .. } = err {
                assert_eq!(reason, "InvalidSignature");
            } else {
                panic!("Expected RevertTransactionError");
            }
        }
    }
}
