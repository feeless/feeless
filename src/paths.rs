//! Filesystem tools for finding out OS independent directory names etc.
use crate::Network;
use clap::Clap;
use directories::BaseDirs;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

/// CLI options for [Paths].
#[derive(Clap)]
pub(crate) struct PathsOpts {
    #[clap(short = 'n', long, default_value = "live")]
    network: Network,

    #[clap(long)]
    data_dir: Option<PathBuf>,
}

impl PathsOpts {
    pub fn wallet_path(&self) -> anyhow::Result<PathBuf> {
        let p = Paths::new_maybe_custom(self.network.clone(), self.data_dir.clone());
        p.ensure_data_path()?;
        Ok(p.wallet_path())
    }
}

/// Contains the base path to wallets, databases, etc.
///
/// It requires the network type so that the path constructed is in this layout:
/// `{app_data}/feeless/{network}/{file or dir}
///
/// For example:
/// * /home/gak/.local/share/feeless/live/wallet.dat
/// * C:\Users\gak\App Data\Local\feeless\live\wallet.dat
pub(crate) struct Paths {
    pub data: PathBuf,
}

impl Paths {
    /// New [Paths] with sane defaults.
    pub fn new(network: Network) -> Self {
        Self {
            data: BaseDirs::new()
                .expect("No HOME environment set.")
                .data_local_dir()
                .join("feeless")
                .join(Self::network_path(network)),
        }
    }

    /// Allow a custom path.
    pub fn new_custom(network: Network, data: PathBuf) -> Self {
        Self {
            data: data.join(Self::network_path(network)),
        }
    }

    /// Optional custom path.
    pub fn new_maybe_custom(network: Network, data: Option<PathBuf>) -> Self {
        match data {
            Some(p) => Self::new_custom(network, p),
            None => Self::new(network),
        }
    }

    fn network_path(network: Network) -> PathBuf {
        network.to_string().to_ascii_lowercase().into()
    }

    /// Join the data path to the specified path. This will be OS dependant,
    /// e.g. in Linux, $HOME/.local/share/feeless/{path}
    pub fn data_path(&self, path: &Path) -> PathBuf {
        self.data.join(path)
    }

    /// Return the path to the wallet.
    pub fn wallet_path(&self) -> PathBuf {
        self.data_path(Path::new("wallet"))
    }

    /// Make sure the data path exists.
    pub fn ensure_data_path(&self) -> anyhow::Result<()> {
        create_dir_all(&self.data)?;
        Ok(())
    }
}
