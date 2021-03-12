use duct::cmd;

fn main() {
    cmd!("feeless", "--help").run().unwrap();
}