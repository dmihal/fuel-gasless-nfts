mod utils;

use fuels::{
    prelude::*,
    tx::{Bytes32, Receipt},
    types::{
        input::Input, output::Output, transaction_builders::ScriptTransactionBuilder,
        transaction_builders::TransactionBuilder, Bits256, ContractId, TxPointer, UtxoId,
    },
};
use utils::{setup, GasPredicateEncoder};

#[tokio::test]
async fn can_use_script() {
    let fixture = setup().await;

    let deployer = &fixture.wallets[0];
    let user = &fixture.wallets[1];
    let user_2 = &fixture.wallets[2];
    let fuel_provider = deployer.provider().unwrap();
    let network_info = fuel_provider.network_info().await.unwrap();

    let predicate = fixture
        .gas_predicate
        .with_data(GasPredicateEncoder::encode_data(vec![], Some(0)));

    let mut inputs = vec![Input::Contract {
        utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
        balance_root: Bytes32::zeroed(),
        state_root: Bytes32::zeroed(),
        tx_pointer: TxPointer::default(),
        contract_id: fixture.nft_instance.id().into(),
    }];

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
        },
    ];

    // Create the Tx
    let transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        inputs,
        outputs,
        TxParameters::default(),
        network_info.clone(),
    )
    .with_script(
        fixture
            .script
            .main(user.address(), false)
            .script_call
            .script_binary,
    )
    .with_script_data(
        fixture
            .script
            .main(user.address(), false)
            .script_call
            .encoded_args
            .resolve(0),
    );

    let mut script_transaction = transaction_builder.build().unwrap();

    let expected_tx_id = script_transaction.id(network_info.chain_id());
    let signature = deployer.sign_message(expected_tx_id).await.unwrap();
    script_transaction.append_witness(signature.as_ref().into());

    let actual_tx_id = fuel_provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .unwrap();
    assert_eq!(expected_tx_id, actual_tx_id);

    let tx_status = fuel_provider.tx_status(&actual_tx_id).await.unwrap();
    let receipts = tx_status.take_receipts_checked(None).unwrap();

    let mut nft_asset_id: Option<AssetId> = None;
    let mut nft_sub_id: Option<Bytes32> = None;
    for item in receipts.iter() {
        match item {
            Receipt::Mint {
                contract_id,
                sub_id,
                ..
            } => {
                let nft_id: ContractId = fixture.nft_instance.id().into();
                assert!(contract_id.clone() == nft_id);
                nft_sub_id = Some(*sub_id);
            }
            Receipt::TransferOut { to, asset_id, .. } => {
                let user_address: Address = user.address().into();
                assert!(to.clone() == user_address);
                nft_asset_id = Some(*asset_id);
            }
            Receipt::LogData { data, .. } => {
                println!("LogData: {}", vec_to_str(&data.clone().unwrap()));
            }
            _ => {}
        }
    }

    // =================
    // Do basic transfer
    // =================

    let predicate = predicate.with_data(GasPredicateEncoder::encode_data(
        vec![Bits256(nft_sub_id.unwrap().into())],
        Some(1),
    ));

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

    let actual_tx_id = fuel_provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .unwrap();
    assert_eq!(expected_tx_id, actual_tx_id);

    let tx_status = fuel_provider.tx_status(&actual_tx_id).await.unwrap();
    let _receipts = tx_status.take_receipts_checked(None).unwrap();
}

#[tokio::test]
async fn can_mint_and_use_packet() {
    let fixture = setup().await;

    let deployer = &fixture.wallets[0];
    let user = &fixture.wallets[1];
    let user_2 = &fixture.wallets[2];
    let fuel_provider = deployer.provider().unwrap();
    let network_info = fuel_provider.network_info().await.unwrap();

    let gas_predicate = fixture
        .gas_predicate
        .with_data(GasPredicateEncoder::encode_data(vec![], Some(0)));

    let mut inputs = vec![
        Input::Contract {
            utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
            tx_pointer: TxPointer::default(),
            contract_id: fixture.nft_instance.id().into(),
        },
        Input::Contract {
            utxo_id: UtxoId::new(Bytes32::zeroed(), 0),
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
            tx_pointer: TxPointer::default(),
            contract_id: fixture.packet_minter_instance.id().into(),
        },
    ];

    let eth_inputs = gas_predicate
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
        Output::Contract {
            input_index: 1u8,
            balance_root: Bytes32::zeroed(),
            state_root: Bytes32::zeroed(),
        },
        Output::Variable {
            to: Address::default(),
            amount: 0,
            asset_id: AssetId::default(),
        },
        Output::Variable {
            to: Address::default(),
            amount: 0,
            asset_id: AssetId::default(),
        },
        Output::Change {
            to: gas_predicate.address().into(),
            amount: 0,
            asset_id: BASE_ASSET_ID,
        },
    ];

    // Create the Tx
    let transaction_builder = ScriptTransactionBuilder::prepare_transfer(
        inputs,
        outputs,
        TxParameters::default(),
        network_info.clone(),
    )
    .with_script(
        fixture
            .script
            .main(user.address(), true)
            .script_call
            .script_binary,
    )
    .with_script_data(
        fixture
            .script
            .main(user.address(), true)
            .script_call
            .encoded_args
            .resolve(0),
    );

    let mut script_transaction = transaction_builder.build().unwrap();

    let expected_tx_id = script_transaction.id(network_info.chain_id());
    let signature = deployer.sign_message(expected_tx_id).await.unwrap();
    script_transaction.append_witness(signature.as_ref().into());

    let actual_tx_id = fuel_provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .unwrap();
    assert_eq!(expected_tx_id, actual_tx_id);

    let transaction = fuel_provider
        .get_transaction_by_id(&actual_tx_id)
        .await
        .unwrap()
        .unwrap();
    let outputs = transaction.transaction.outputs();
    let nft = outputs[2];
    let packet = outputs[3];

    let expected_packet_id = fixture
        .packet_minter_instance
        .id()
        .asset_id(&Bits256(*fixture.user.address().hash()));

    assert_eq!(packet.amount().unwrap(), 1);
    assert_eq!(packet.asset_id().unwrap().clone(), expected_packet_id);
    assert_eq!(
        packet.to().unwrap().clone(),
        fixture.packet_predicate.address().into()
    );

    // =================
    // Do basic transfer
    // =================

    let gas_predicate = gas_predicate.with_data(GasPredicateEncoder::encode_data(
        vec![Bits256([0; 32])],
        None,
    ));

    let mut nft_inputs = user
        .get_asset_inputs_for_amount(nft.asset_id().unwrap().clone(), 1)
        .await
        .unwrap();

    let packet_inputs = fixture
        .packet_predicate
        .get_asset_inputs_for_amount(expected_packet_id, 1)
        .await
        .unwrap();

    let eth_inputs = gas_predicate
        .get_asset_inputs_for_amount(BASE_ASSET_ID, 1000)
        .await
        .unwrap();

    nft_inputs.extend(packet_inputs);
    nft_inputs.extend(eth_inputs);

    let outputs = vec![
        Output::Change {
            to: gas_predicate.address().into(),
            amount: 0,
            asset_id: BASE_ASSET_ID,
        },
        Output::Coin {
            to: fixture.packet_predicate.address().into(),
            amount: 1,
            asset_id: expected_packet_id,
        },
        Output::Coin {
            to: user_2.address().into(),
            amount: 1,
            asset_id: nft.asset_id().unwrap().clone(),
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

    let script_transaction = transaction_builder.build().unwrap();

    let actual_tx_id = fuel_provider
        .send_transaction_and_await_commit(script_transaction)
        .await
        .unwrap();

    let tx_status = fuel_provider.tx_status(&actual_tx_id).await.unwrap();
    tx_status.check(None).unwrap();
}

fn vec_to_str(vec: &Vec<u8>) -> String {
    vec.iter().map(|b| format!("{:02x}", b)).collect()
}
