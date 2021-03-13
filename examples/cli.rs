use ansi_term::Color;

use clap::Clap;
use duct::cmd;

use std::path::PathBuf;

// These Opts are only for the path to feeless, not the actual feeless CLI.
#[derive(Clap)]
struct Opts {
    /// Specify the path to the feeless binary, e.g. "target/Debug/feeless"
    feeless_path: PathBuf,
}

/// This example is to show how to use the CLI, and also acts as an integration test.
fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let bin = &opts.feeless_path;

    let mut test = Test::new();

    test.assert("Correctly display the help screen.", || {
        Ok(cmd!(bin, "--help").read()?.contains("cryptocurrency"))
    });

    test.assert(
        "A new phrase piped through several stages into an address.",
        || {
            Ok(cmd!(bin, "phrase", "new")
                .pipe(cmd!(bin, "phrase", "private", "-"))
                .pipe(cmd!(bin, "private", "public", "-"))
                .pipe(cmd!(bin, "public", "address", "-"))
                .read()?
                .contains("nano_"))
        },
    );

    let phrase = "讓 步 械 遞 窮 針 柳 擾 逃 湯 附 剛";
    // This is address 5 from the phrase.
    let addr = "nano_3tr7wk6ebc6ujptdnf471d8knnfaz1r469u83biws5s5jntb3hpe8oh65ogi";
    test.assert("A known phrase converted directly to an address.", || {
        Ok(
            cmd!(bin, "phrase", "address", "-l", "zh-hant", "-a", "5", phrase)
                .read()?
                .contains(addr),
        )
    });

    test.assert(
        "A known phrase piped through several stages into an address.",
        || {
            Ok(
                cmd!(bin, "phrase", "private", "-l", "zh-hant", "-a", "5", phrase)
                    .pipe(cmd!(bin, "private", "public", "-"))
                    .pipe(cmd!(bin, "public", "address", "-"))
                    .read()?
                    .contains(addr),
            )
        },
    );

    test.assert(
        "A known seed piped through several stages into an address.",
        || {
            let zeros = "0000000000000000000000000000000000000000000000000000000000000000";
            Ok(cmd!(bin, "seed", "private", zeros, "-i", "0")
                .pipe(cmd!(bin, "private", "address", "-"))
                .read()?
                .contains("nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"))
        },
    );

    Ok(())
}

/// A basic testing suite, allowing to continue after a failure.
struct Test {
    has_failed: bool,
}

impl Test {
    fn new() -> Self {
        Self { has_failed: false }
    }

    fn assert<F>(&mut self, desc: &str, result: F)
    where
        F: Fn() -> anyhow::Result<bool>,
    {
        let (ok, msg) = match result() {
            Ok(r) => match r {
                true => (true, Color::Green.bold().paint("PASS")),
                false => (false, Color::Red.bold().paint("FAIL")),
            },
            Err(_err) => (false, Color::Red.bold().paint("ERRO")),
        };
        println!("{} {}", msg, desc);
        if !ok {
            self.has_failed = true;
        }
    }
}
