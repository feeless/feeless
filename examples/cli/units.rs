use crate::Test;
use cmd_lib::run_fun;

pub fn units(test: &mut Test, feeless: &str) -> anyhow::Result<()> {
    test.run("Convert Mnano to raw.", || {
        Ok(run_fun!(
            $feeless unit mnano raw 1
        )?)
    })
    .equals("1000000000000000000000000000000");

    test.run("Convert nano to raw.", || {
        Ok(run_fun!(
            $feeless unit nano raw 1
        )?)
    })
    .equals("1000000000000000000000000");

    test.run("Convert raw to nano.", || {
        Ok(run_fun!(
            $feeless unit raw nano 1
        )?)
    })
    .equals("0.000000000000000000000001");

    test.run("Convert raw to Mnano.", || {
        Ok(run_fun!(
            $feeless unit raw mnano 1
        )?)
    })
    .equals("0.000000000000000000000000000001");

    Ok(())
}
