
// Network calls (server.load_account, server.submit_transaction) are
// commented out and replaced with prints, since they require a live
// connection to a real Horizon server to actually execute. Everything
// else, keypair generation, asset creation, operation building,
// transaction assembly, signing, and XDR encoding, runs for real.

use stellar_base::asset::Asset;
use stellar_base::operations::Operation;
use stellar_base::transaction::{Transaction, MIN_BASE_FEE};
use stellar_base::network::Network;
use stellar_base::amount::Amount;
use stellar_base::amount::Stroops;
use stellar_base::crypto::{DalekKeyPair, PublicKey};
use stellar_base::xdr::XDRSerialize;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Dummy values, standing in for what would normally come from
    // your own account setup and a Horizon query

    // FIX for error 3: source_kp was never defined, generate one
    let source_kp = DalekKeyPair::random()?;

    // FIX for error 1: issuer was never defined, generate a second
    // random keypair and use its public key as a stand-in issuer
    let issuer_kp = DalekKeyPair::random()?;
    let issuer: PublicKey = issuer_kp.public_key().clone();

    // FIX for error 3: sequence was never defined, a real value would
    // come from server.load_account(...).sequence_number(), 1 is a
    // placeholder here since there is no live account behind this
    let sequence: i64 = 1;

    println!("Source account   = {}", source_kp.public_key().account_id());
    println!("Issuer account   = {}", issuer.account_id());

    // 1. Build the ChangeTrust operation

    let credit_asset = Asset::new_credit("USDC", issuer)?;

    let trust_op = Operation::new_change_trust()
        .with_asset(credit_asset.clone().into())
        // FIX for error 2: None::<&str> failed because &str does not
        // convert into Stroops. None::<i64> does, since Stroops wraps
        // a plain integer internally.
        .with_limit(Some(Stroops::max()))?
        .build()?;

    println!("Built ChangeTrust operation for asset USDC");

    // 2. Assemble and sign the trustline transaction

    let mut trust_tx = Transaction::builder(
        source_kp.public_key(),
        sequence,
        MIN_BASE_FEE,
    )
        .add_operation(trust_op)
        .into_transaction()?;

    trust_tx.sign(&source_kp.as_ref(), &Network::new_test());

    let trust_xdr = trust_tx.into_envelope().xdr_base64()?;
    println!("Signed trustline transaction XDR:\n{}", trust_xdr);

    // In real code, this line submits it to Horizon:
    // let trust_response = server.submit_transaction(trust_tx)?;
    println!("(submission skipped, no live server connection in this example)");

    // 3. Build and sign the payment

    // Stand-in recipient, a third random keypair
    let destination_kp = DalekKeyPair::random()?;
    let destination: PublicKey = destination_kp.public_key().clone();

    let send_amount = Amount::from_str("25")?;

    let payment_op = Operation::new_payment()
        .with_destination(destination)
        .with_amount(send_amount)?
        .with_asset(credit_asset)
        .build()?;

    println!("Built Payment operation for 25 USDC");

    // A real payment transaction needs a fresh sequence number, since
    // the trustline transaction above would have consumed the old one.
    // Here we just increment the placeholder by 1 to simulate that.
    let payment_sequence: i64 = sequence + 1;

    let mut payment_tx = Transaction::builder(
        source_kp.public_key(),
        payment_sequence,
        MIN_BASE_FEE,
    )
        .add_operation(payment_op)
        .into_transaction()?;

    payment_tx.sign(&source_kp.as_ref(), &Network::new_test());

    let payment_xdr = payment_tx.into_envelope().xdr_base64()?;
    println!("Signed payment transaction XDR:\n{}", payment_xdr);

    // In real code, this line submits it to Horizon:
    // server.submit_transaction(payment_tx)?;
    println!("(submission skipped, no live server connection in this example)");

    Ok(())
} fn main
