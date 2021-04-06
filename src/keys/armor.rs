use crate::{Address, Error, Result, Signature};
use std::fmt::{Display, Formatter};
use std::str::{FromStr, Split};

#[derive(Debug)]
pub struct Armor {
    message: String,
    address: Address,
    signature: Signature,
}

impl Armor {
    const BEGIN_MESSAGE: &'static str = "-----BEGIN NANO SIGNED MESSAGE-----";
    const BEGIN_ADDRESS: &'static str = "-----BEGIN NANO ADDRESS-----";
    const BEGIN_SIGNATURE: &'static str = "-----BEGIN NANO SIGNATURE-----";
    const END_SIGNATURE: &'static str = "-----END NANO SIGNATURE-----";

    pub fn new(message: String, address: Address, signature: Signature) -> Self {
        Self {
            message,
            address,
            signature,
        }
    }

    pub fn verify(&self) -> Result<()> {
        self.address
            .to_public()
            .verify(self.message.as_bytes(), &self.signature)
    }
}

impl Display for Armor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(Self::BEGIN_MESSAGE)?;
        f.write_str("\n")?;
        f.write_str(&self.message.to_string())?;
        f.write_str("\n")?;
        f.write_str(Self::BEGIN_ADDRESS)?;
        f.write_str("\n")?;
        f.write_str(&self.address.to_string())?;
        f.write_str("\n")?;
        f.write_str(Self::BEGIN_SIGNATURE)?;
        f.write_str("\n")?;
        f.write_str(&self.signature.to_string())?;
        f.write_str("\n")?;
        f.write_str(Self::END_SIGNATURE)
    }
}

impl FromStr for Armor {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut iter = s.split("\n");

        decode_static(Self::BEGIN_MESSAGE, iter.next(), "begin message")?;
        let message = decode_part(&mut iter, "Missing message")?;

        decode_static(Self::BEGIN_ADDRESS, iter.next(), "begin address")?;
        let address_str = decode_part(&mut iter, "Missing address")?;
        let address = Address::from_str(&address_str)?;

        decode_static(Self::BEGIN_SIGNATURE, iter.next(), "begin signature")?;
        let signature_str = decode_part(&mut iter, "Missing signature")?;
        let signature = Signature::from_str(&signature_str)?;

        decode_static(Self::END_SIGNATURE, iter.next(), "end signature")?;

        Ok(Self::new(message, address, signature))
    }
}

fn decode_static(expected: &str, got: Option<&str>, what: &str) -> Result<()> {
    if let Some(begin) = got {
        let begin = begin.trim();
        if begin != expected {
            return Err(Error::InvalidArmor(format!(
                "Incorrect {}: Expecting: {} Got: {}",
                what, expected, begin
            )));
        }
    } else {
        return Err(Error::InvalidArmor(format!("Missing {}", what)));
    }

    Ok(())
}

fn decode_part(iter: &mut Split<&str>, what: &str) -> Result<String> {
    Ok(iter
        .next()
        .ok_or_else(|| Error::InvalidArmor(what.into()))?
        .trim()
        .to_owned())
}
