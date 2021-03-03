use regex;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

// Mar 03 11:24:17.166 DEBUG feeless::node::controller: Packet: #377 2021-02-26T00:46:42.042497+00:00
// >>> 165.227.25.198:7075 size: 40 Handshake { query: Some(HandshakeQuery(Cookie(DFD998DCA7 ...
pub fn parse_pcap_log_file_to_csv(input: &Path, output: &Path) -> anyhow::Result<()> {
    let input = File::open(input)?;
    let re = Regex::new("Packet: #(?P<packet>[0-9]+).*size: (?P<size>[0-9]+) (?P<msg>[a-zA-z]+)")
        .unwrap();
    let mut accum = HashMap::new();

    for line in BufReader::new(input).lines() {
        let line = line?;
        for cap in re.captures_iter(&line) {
            let packet = match cap.name("packet") {
                Some(p) => p,
                None => continue,
            };
            let size = match cap.name("size") {
                Some(p) => p,
                None => continue,
            };
            let msg = match cap.name("msg") {
                Some(p) => p,
                None => continue,
            };

            let msg = msg.as_str();
            let a = accum.entry(msg.to_owned()).or_insert(0);
            *a += 1;
            println!("{},{},{},{}", packet.as_str(), size.as_str(), &msg, &a);
        }
    }
    Ok(())
}
