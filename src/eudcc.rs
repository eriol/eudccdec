use std::collections::BTreeMap;
use std::io::Read;

use anyhow::{bail, Result};
use base45;
use flate2::read::ZlibDecoder;
use serde_cbor::value::from_value;
use serde_cbor::{from_slice, Value};
use serde_derive::Deserialize;

const HC1_FIELD: &str = "HC1:";
const HCERT_CLAIM_KEY: i128 = -260;
const DCC: i128 = 1;

#[derive(Debug, Deserialize)]
struct Vaccine {
    ci: String,
    co: String,
    dn: i32,
    dt: String,
    is: String,
    ma: String,
    mp: String,
    sd: i32,
    tg: String,
    vp: String,
}

#[derive(Debug, Deserialize)]
struct Name {
    #[serde(rename = "fn")]
    fn_: String,
    fnt: String,
    gn: String,
    gnt: String,
}

#[derive(Debug, Deserialize)]
pub struct Certificate {
    nam: Name,
    dob: String,
    v: Vec<Vaccine>,
    ver: String,
}

pub fn decode(data: String) -> Result<Certificate> {
    let data = data.trim_end().strip_prefix(HC1_FIELD);

    let base45_data: String = match data {
        Some(data) => data.into(),
        None => bail!("data must start with {} prefix", HC1_FIELD),
    };

    let base45_decoded = base45::decode(&base45_data)?;

    let mut zlibdecoder = ZlibDecoder::new(base45_decoded.as_slice());
    let mut cbor_data = Vec::new();
    zlibdecoder.read_to_end(&mut cbor_data)?;

    let (_header1, _header2, payload, _signature): (
        &[u8],
        BTreeMap<String, Value>,
        &[u8],
        &[u8],
    ) = from_slice(&cbor_data)?;
    let payload: Value = from_slice(&payload)?;

    if let Value::Map(m) = payload {
        if let Some(health_certificate) =
            m.get(&Value::Integer(HCERT_CLAIM_KEY))
        {
            if let Value::Map(m) = health_certificate {
                if let Some(eudccv1) = m.get(&Value::Integer(DCC)) {
                    let cert: Certificate = from_value(eudccv1.clone())?;
                    return Ok(cert);
                }
            }
        }
    }

    bail!("Can't decode the EU Digital COVID Certificate payload!");
}
