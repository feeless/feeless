use anyhow::Result;
/// This example is to show off how to use the CLI, and also acts as an integration test.
use clap::Clap;
use duct::cmd;
use std::path::PathBuf;

// These Opts are only for the path to feeless, not the actual feeless CLI.
#[derive(Clap)]
struct Opts {
    /// Specify the path to the feeless binary, e.g. "target/Debug/feeless"
    feeless_path: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let bin = &opts.feeless_path;

    assert!(cmd!(bin, "--help").read()?.contains("cryptocurrency"));

    // This is just a sanity check since it is non-deterministic. We can't check the result, only
    // to see if there is an address and no error.
    assert!(cmd!(bin, "phrase", "new")
        .pipe(cmd!(bin, "phrase", "public", "-"))
        .pipe(cmd!(bin, "public", "address", "-"))
        .read()?
        .contains("nano_"));

    let phrase = "讓 步 械 遞 窮 針 柳 擾 逃 湯 附 剛";
    assert!(
        cmd!(bin, "phrase", "address", "-l", "zh-hant", "-a", "5", phrase)
            .read()?
            .contains("nano_3tr7wk6ebc6ujptdnf471d8knnfaz1r469u83biws5s5jntb3hpe8oh65ogi")
    );

    Ok(())
}
