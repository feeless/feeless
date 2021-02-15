use crate::{Address, BlockHash, Link, Public, Raw, Signature, StateBlock, Work};
use anyhow::anyhow;
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Network {
    Test = 0x41,
    Beta = 0x42,
    Live = 0x43,
}

impl Network {
    fn genesis_block(&self) -> anyhow::Result<StateBlock> {
        Ok(match self {
            Self::Live => StateBlock::new(
                Address::from_str(
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
                )?
                    .to_public(),
                BlockHash::zero(),
                Address::from_str(
                    "nano_3t6k35gi95xu6tergt6p69ck76ogmitsa8mnijtpxm9fkcm736xtoncuohr3",
                )?
                .to_public(),
                Raw::zero(),
                Link::unsure_from_hex("E89208DD038FBB269987689621D52292AE9C35941A7484756ECCED92A65093BA")?,
                Signature::from_hex("9F0C933C8ADE004D808EA1985FA746A7E95BA2A38F867640F53EC8F180BDFE9E2C1268DEAD7C2664F356E37ABA362BC58E46DBA03E523A7B5A19E4B6EB12BB02")?,
                Work::from_hex("62f05417dd3fb691")?,
            ),
            _ => todo!(),
        })
    }

    fn genesis_hash(&self) -> BlockHash {
        match self {
            Self::Live => BlockHash::from_hex(
                "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
            )
            .unwrap(),
            _ => todo!(),
        }
    }

    fn preconfigured_representatives(&self) -> anyhow::Result<Vec<Public>> {
        todo!()
        // match Network {
        //     case Self::Live => {},
        //     _ => todo!(),
        // }
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
    fn live() {
        let net = Network::Live;
        let block = net.genesis_block().unwrap();
        dbg!(&block);
        let hash = block.hash_as_open().unwrap();
        assert_eq!(hash, net.genesis_hash());
    }
}
