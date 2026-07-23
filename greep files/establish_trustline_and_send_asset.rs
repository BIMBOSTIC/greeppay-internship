

use stellar_base::asset::Asset;
use stellar_base::operations::Operation;
use stellar_base::transaction::{Transaction, MIN_BASE_FEE};
use stellar_base::network::Network;
use stellar_base::amount::Amount;
use stellar_base::crypto::PublicKey;
use std::str::FromStr;

fn establish_trustline_and_send_asset() -> Result<(), Box<dyn std::error::Error>> {

    // 1. Build the ChangeTrust operation

    // Identify the asset: pairs an asset code with its issuer's public key
    let credit_asset = Asset::new_credit("USDC", issuer)?;

    let trust_op = Operation::new_change_trust()
        // .into() converts Asset into the ChangeTrustAsset type the builder expects
        .with_asset(credit_asset.clone().into())
        // None = trust up to the maximum possible limit
        .with_limit(None::<&str>)?
        .build()?;

    // 2. Assemble, sign, and submit the trustline transaction

    let mut trust_tx = Transaction::builder(
        source_kp.public_key(),
        sequence,
        MIN_BASE_FEE,
    )
        .add_operation(trust_op)
        .into_transaction()?; // mut: signing modifies this in place

    // Signs for testnet specifically, network passphrase is part of what gets signed
    trust_tx.sign(&source_kp.as_ref(), &Network::new_test());

    // submit_transaction (stellar_sdk) takes the stellar-base Transaction directly,
    // no manual XDR conversion needed, handled internally
    let _trust_response = server.submit_transaction(trust_tx)?;

    // IMPORTANT: must wait for this to actually confirm on the ledger before
    // building the payment below, otherwise it fails with op_no_trust

    // 3. Build and send the payment

    let destination = PublicKey::from_account_id("G...RECIPIENT")?;
    let send_amount = Amount::from_str("25")?;

    let payment_op = Operation::new_payment()
        .with_destination(destination)
        .with_amount(send_amount)?
        // reuses the same credit asset from step 1, this is the only difference
        // versus sending native XLM (which would use Asset::new_native() instead)
        .with_asset(credit_asset)
        .build()?;

    // sequence numbers only increase by 1 per transaction, the trustline tx
    // above already consumed the old one, so the account must be reloaded
    let account_id = source_kp.public_key().account_id();
    let payment_sequence: i64 = server
        .load_account(&account_id)?
        .sequence_number()
        .parse()?;

    let mut payment_tx = Transaction::builder(
        source_kp.public_key(),
        payment_sequence,
        MIN_BASE_FEE,
    )
        .add_operation(payment_op)
        .into_transaction()?;

    payment_tx.sign(&source_kp.as_ref(), &Network::new_test());
    server.submit_transaction(payment_tx)?;

    Ok(())
}
