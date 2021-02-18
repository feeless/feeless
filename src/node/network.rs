use std::convert::TryFrom;

use anyhow::anyhow;

use crate::blocks::FullBlock;
use crate::blocks::Link;
use crate::blocks::OpenBlock;
use crate::encoding::FromHex;
use crate::{Address, BlockHash, Public, Raw, Signature, Work};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Network {
    Test = 0x41,
    Beta = 0x42,
    Live = 0x43,
}

fn live_genesis_block() -> FullBlock {
    serde_json::from_str(
    r#"
        {
            "type": "open",
            "source": "E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA",
            "representative": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
            "account": "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
            "work": "62F05417DD3FB691",
            "signature": "9F0C933C8ADE004D808EA1985FA746A7E95BA2A38F867640F53EC8F180BDFE9E2C1268DEAD7C2664F356E37ABA362BC58E46DBA03E523A7B5A19E4B6EB12BB02"
        }
        "#
    ).unwrap()
}

impl Network {
    pub fn genesis_block(&self) -> FullBlock {
        match self {
            Self::Live => live_genesis_block(),
            _ => todo!(),
        }
    }

    pub fn genesis_hash(&self) -> BlockHash {
        match self {
            Self::Live => BlockHash::from_hex(
                "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
            )
            .unwrap(),
            _ => todo!(),
        }
    }

    pub fn genesis_account(&self) -> Public {
        let block = self.genesis_block();
        let open = block.open_block().unwrap();
        open.account.to_owned()
    }
}

impl TryFrom<u8> for Network {
    type Error = anyhow::Error;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        use Network::*;
        Ok(match v {
            0x41 => Test,
            0x42 => Beta,
            0x43 => Live,
            v => return Err(anyhow!("Unknown network: {} ({:X})", v, v)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_live_genesis_block() {
        let net = Network::Live;
        let block = net.genesis_block();
        let hash = block.hash().unwrap();
        assert_eq!(hash, net.genesis_hash());
    }
}
