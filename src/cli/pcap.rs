use anyhow::Context;
use clap::Clap;
use std::net::Ipv4Addr;
use std::str::FromStr;

/// Read a pcapng file containing Nano packets, and print some information about each payload.
#[derive(Clap)]
pub(crate) struct PcapDumpOpts {
    path: String,

    /// The IP address of the subject, to show relative information.
    /// This is inferred automatically by the host of the first packet sent out.
    #[clap(short, long)]
    my_addr: Option<String>,

    /// Only show packets related to this IP address.
    #[clap(long)]
    filter_addr: Option<String>,

    /// Starting packet.
    #[clap(long)]
    start: Option<usize>,

    /// Last packet to process.
    #[clap(long)]
    end: Option<usize>,
}

impl PcapDumpOpts {
    pub async fn handle(&self) -> anyhow::Result<()> {
        let subject = match &self.my_addr {
            Some(ip_addr) => crate::pcap::Subject::Specified(
                Ipv4Addr::from_str(&ip_addr).context("Invalid IP address")?,
            ),
            None => crate::pcap::Subject::AutoFirstSource,
        };
        let mut p = crate::pcap::PcapDump::new(subject);
        p.start_at = self.start;
        p.end_at = self.end;
        p.filter_addr = self
            .filter_addr
            .as_ref()
            .map(|i| Ipv4Addr::from_str(i).context("Invalid IP address"))
            .transpose()?;
        p.dump(&self.path).await
    }
}
