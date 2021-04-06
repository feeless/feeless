use crate::Test;
use cmd_lib::run_fun;
use std::env::set_var;
use std::fs::remove_file;
use std::path::Path;

pub fn signing(test: &mut Test, feeless: &str) -> anyhow::Result<()> {
    let wallet_path = "test.wallet";
    if Path::new(wallet_path).exists() {
        remove_file(wallet_path)?;
    }
    set_var("FEELESS_WALLET_FILE", wallet_path);

    let wallet_id = run_fun!(
        $feeless wallet new phrase
    )?;
    set_var("FEELESS_WALLET_ID", wallet_id);

    let address = run_fun!(
        $feeless wallet address
    )?;

    let signature = test
        .run("Sign a message", || {
            Ok(run_fun!(
                $feeless wallet sign "secret message"
            )?)
        })
        .output;

    test.run("Verify a message", || {
        Ok(run_fun!(
            $feeless verify --message "secret message" --address $address --signature $signature
        )?)
    });

    let armor_path = "test.armor";
    test.run("Sign a message with armor output", || {
        Ok(run_fun!(
            $feeless wallet sign "another message" --armor > $armor_path
        )?)
    });

    test.run("Verify an armor message", || {
        Ok(run_fun!(
            $feeless verify --armor < $armor_path
        )?)
    });

    remove_file(armor_path)?;
    remove_file(wallet_path)?;

    Ok(())
}
