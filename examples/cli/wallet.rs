use crate::Test;
use cmd_lib::run_fun;
use std::env::set_var;
use std::fs::remove_file;
use std::path::Path;

pub(crate) fn wallet(test: &mut Test, feeless: &str) -> anyhow::Result<()> {
    let wallet_path = "test.wallet";
    if Path::new(wallet_path).exists() {
        remove_file(wallet_path)?;
    }

    let outcome = test.run("A new wallet with a random phrase.", || {
        Ok(run_fun!(
            $feeless wallet new phrase --file test.wallet
        )?)
    });
    let wallet_id = outcome.output;

    test.run("Generate an address from a phrase wallet.", || {
        Ok(run_fun!(
            $feeless wallet address --file $wallet_path --id $wallet_id
        )?)
    })
    .contains("nano_");

    test.run("Delete wallet.", || {
        Ok(run_fun!(
            $feeless wallet delete --file $wallet_path --id $wallet_id
        )?)
    });

    test.run(
        "Import into a the default wallet with a known phrase.",
        || {
            Ok(run_fun!(
                $feeless wallet import phrase --file $wallet_path --default --language zh-hant "讓 步 械 遞 窮 針 柳 擾 逃 湯 附 剛"
            )?)
        },
    ).contains("0000");

    set_var("FEELESS_WALLET_FILE", wallet_path);
    test.run(
        "Generate an address from the default wallet and access an environment variable.",
        || {
            Ok(run_fun!(
                $feeless wallet address 5
            )?)
        },
    )
    .contains("nano_3tr7wk6ebc6ujptdnf471d8knnfaz1r469u83biws5s5jntb3hpe8oh65ogi");

    remove_file(wallet_path)?;

    Ok(())
}
