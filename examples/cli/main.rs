#![forbid(unsafe_code)]
#![cfg_attr(feature = "deny_warnings", deny(warnings))]

mod keys;
mod signing;
mod units;
mod wallet;

use ansi_term::Color;
use anyhow::{anyhow, Context};
use clap::Clap;
use cmd_lib::log::LevelFilter;
use cmd_lib::run_fun;
use std::ffi::OsStr;
use std::fs::remove_dir_all;
use std::ops::Deref;
use std::path::Path;
use std::process::exit;

// These `Opts` are only for the path to feeless, not the actual feeless CLI.
#[derive(Clap)]
struct Opts {
    /// Specify the path to the feeless binary, e.g. "target/Debug/feeless"
    feeless_path: String,
}

/// This example is to show how to use the CLI, and also acts as an integration test.
fn main() {
    if let Err(err) = run() {
        println!("Error: {:?}", err);
        exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Trace)
        .init();

    let opts = Opts::parse();
    let feeless = &opts.feeless_path;
    let mut test = Test::new();

    test.run("Display the help screen.", || {
        Ok(run_fun!(
            $feeless --help
        )?)
    })
    .contains("cryptocurrency");

    keys::keys(&mut test, &feeless)?;
    wallet::wallet(&mut test, &feeless)?;
    signing::signing(&mut test, &feeless)?;
    units::units(&mut test, &feeless)?;

    test.end()?;

    Ok(())
}

/// A basic testing suite, allowing to continue after a failure.
pub struct Test {
    outcomes: Vec<Outcome>,
}

impl Test {
    fn new() -> Self {
        Self { outcomes: vec![] }
    }

    fn run<F>(&mut self, desc: &str, result: F) -> Outcome
    where
        F: Fn() -> anyhow::Result<String>,
    {
        let outcome = match result() {
            Ok(s) => Outcome::new(desc, &s, State::Pass),
            Err(err) => Outcome::new(desc, "", State::Error(format!("{:?}", err))),
        };
        self.outcomes.push(outcome.to_owned());
        outcome
    }

    fn end(&self) -> anyhow::Result<()> {
        for o in &self.outcomes {
            if o.state != State::Pass {
                return Err(anyhow!("Failed tests."));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq)]
enum State {
    Pass,
    Error(String),
    Fail(String),
}

#[derive(Clone)]
struct Outcome {
    desc: String,
    output: String,
    state: State,
}

impl Outcome {
    pub fn new(desc: &str, output: &str, state: State) -> Self {
        Self {
            desc: desc.to_owned(),
            output: output.to_owned(),
            state,
        }
    }

    pub fn contains(&mut self, s: &str) -> &mut Self {
        if let State::Error(_) = &self.state {
            self.print();
            return self;
        }

        if !self.output.contains(s) {
            self.state = State::Fail(format!(
                "Output does not contain '{}'.\nOutput: {}",
                s, &self.output
            ));
            self.print();
            return self;
        }

        self.print();
        self
    }

    pub fn equals(&mut self, s: &str) -> &mut Self {
        if let State::Error(_) = &self.state {
            self.print();
            return self;
        }

        if !self.output.eq(s) {
            self.state = State::Fail(format!(
                "Output does not equal '{}'.\nOutput: {}",
                s, &self.output
            ));
            self.print();
            return self;
        }

        self.print();
        self
    }

    pub fn print(&self) {
        let (prefix, message) = match &self.state {
            State::Error(err) => (Color::Red.bold().paint("ERRO"), Some(err.to_owned())),
            State::Fail(msg) => (Color::Red.bold().paint("FAIL"), Some(msg.to_owned())),
            State::Pass => (Color::Green.bold().paint("PASS"), None),
        };

        println!("{} {}", prefix, self.desc);
        if let Some(err) = message {
            println!("{}", Color::Purple.paint(err));
        }
    }
}

#[derive(Debug)]
struct DataDir(String);

impl Drop for DataDir {
    fn drop(&mut self) {
        if Path::new(&self.0).exists() {
            if let Err(err) = remove_dir_all(&self.0).context("Removing existing data dir") {
                eprintln!("Error dropping DataDir: {:?}", err);
            }
        }
    }
}

impl Deref for DataDir {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<OsStr> for DataDir {
    fn as_ref(&self) -> &OsStr {
        &OsStr::new(&self.0)
    }
}

fn setup_data_dir() -> anyhow::Result<DataDir> {
    let data_dir = "examples-data-dir";
    if Path::new(data_dir).exists() {
        remove_dir_all(data_dir).context("Removing existing data dir")?;
    }
    Ok(DataDir(data_dir.into()))
}
