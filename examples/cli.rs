use ansi_term::Color;
use clap::Clap;
use cmd_lib::{run_fun};


// These `Opts` are only for the path to feeless, not the actual feeless CLI.
#[derive(Clap)]
struct Opts {
    /// Specify the path to the feeless binary, e.g. "target/Debug/feeless"
    feeless_path: String,
}

/// This example is to show how to use the CLI, and also acts as an integration test.
fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let feeless = &opts.feeless_path;

    let mut test = Test::new();

    test.assert("Display the help screen.", || {
        Ok(run_fun!(
            $feeless --help
        )?
        .contains("cryptocurrency"))
    });

    test.assert(
        "A new phrase piped through several stages into an address.",
        || {
            Ok(run_fun!(
                $feeless phrase new |
                $feeless phrase to-private - |
                $feeless private to-public - |
                $feeless public to-address -
            )?
            .contains("nano_"))
        },
    );

    // Example from https://docs.nano.org/integration-guides/key-management/#test-vectors
    test.assert("A phrase with a passphrase converted to an address.", || {
        Ok(run_fun!(
            $feeless phrase to-address -p "some password"
            "edge defense waste choose enrich upon flee junk siren film clown finish luggage leader kid quick brick print evidence swap drill paddle truly occur"
        )?
        .contains("nano_1pu7p5n3ghq1i1p4rhmek41f5add1uh34xpb94nkbxe8g4a6x1p69emk8y1d"))
    });

    // A zh-hant phrase
    let phrase = "讓 步 械 遞 窮 針 柳 擾 逃 湯 附 剛";

    // This is address 5 from the phrase.
    let addr = "nano_3tr7wk6ebc6ujptdnf471d8knnfaz1r469u83biws5s5jntb3hpe8oh65ogi";

    test.assert("A phrase converted directly to an address.", || {
        Ok(run_fun!(
            $feeless phrase to-address -l zh-hant -a 5 "$phrase"
        )?
        .contains(addr))
    });

    test.assert(
        "A phrase piped through several stages into an address.",
        || {
            Ok(run_fun!(
                $feeless phrase to-private -l zh-hant -a 5 "$phrase" |
                $feeless private to-public - |
                $feeless public to-address -
            )?
            .contains(addr))
        },
    );

    test.assert("A seed directly converted to a public key.", || {
        let zeros = "0000000000000000000000000000000000000000000000000000000000000000";
        Ok(run_fun!(
            $feeless seed to-public $zeros -i 0
        )?
        .contains("C008B814A7D269A1FA3C6528B19201A24D797912DB9996FF02A1FF356E45552B"))
    });

    test.assert("A seed directly converted to an address.", || {
        let zeros = "0000000000000000000000000000000000000000000000000000000000000000";
        Ok(run_fun!(
            $feeless seed to-address $zeros -i 0
        )?
        .contains("nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"))
    });

    test.assert("A seed piped through to convert it to an address.", || {
        let zeros = "0000000000000000000000000000000000000000000000000000000000000000";
        Ok(run_fun!(
            $feeless seed to-private $zeros -i 0 | $feeless private to-address -
        )?
        .contains("nano_3i1aq1cchnmbn9x5rsbap8b15akfh7wj7pwskuzi7ahz8oq6cobd99d4r3b7"))
    });

    test.assert("A random private key into an address.", || {
        Ok(run_fun!(
            $feeless private new | $feeless private to-address -
        )?
        .contains("nano_"))
    });

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
        let (ok, msg, maybe_err) = match result() {
            Ok(r) => match r {
                true => (true, Color::Green.bold().paint("PASS"), None),
                false => (false, Color::Red.bold().paint("FAIL"), None),
            },
            Err(err) => (false, Color::Red.bold().paint("ERRO"), Some(err)),
        };
        println!("{} {}", msg, desc);
        if let Some(err) = maybe_err {
            println!("{:?}", err);
        }
        if !ok {
            self.has_failed = true;
        }
    }
}
