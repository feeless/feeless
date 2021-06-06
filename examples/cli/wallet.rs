use crate::{setup_data_dir, Test};
use cmd_lib::run_fun;

pub(crate) fn wallet(test: &mut Test, feeless: &str) -> anyhow::Result<()> {
    let data_dir = setup_data_dir()?;

    let outcome = test.run("A new wallet with a random phrase.", || {
        Ok(run_fun!(
            $feeless wallet new phrase --data-dir $data_dir
        )?)
    });
    let wallet_id = outcome.output;

    test.run("Generate an address from a phrase wallet.", || {
        Ok(run_fun!(
            $feeless wallet address --data-dir $data_dir --id $wallet_id
        )?)
    })
    .contains("nano_");

    test.run("Delete wallet.", || {
        Ok(run_fun!(
            $feeless wallet delete --data-dir $data_dir --id $wallet_id
        )?)
    });

    test.run(
        "Import into a the default wallet with a known phrase.",
        || {
            Ok(run_fun!(
                $feeless wallet import phrase --data-dir $data_dir --default --language zh-hant "讓 步 械 遞 窮 針 柳 擾 逃 湯 附 剛"
            )?)
        },
    ).contains("0000");

    test.run(
        "Generate an address from the default wallet and access an environment variable.",
        || {
            Ok(run_fun!(
                FEELESS_DATA_DIR=$data_dir $feeless wallet address 5
            )?)
        },
    )
    .contains("nano_3tr7wk6ebc6ujptdnf471d8knnfaz1r469u83biws5s5jntb3hpe8oh65ogi");

    Ok(())
}
