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

#[derive(Debug, Deserialize, PartialEq)]
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

#[derive(Debug, Deserialize, PartialEq)]
struct Recovery {
    tg: String,
    fr: String,
    co: String,
    is: String,
    df: String,
    du: String,
    ci: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Name {
    #[serde(rename = "fn")]
    fn_: String,
    fnt: String,
    gn: String,
    gnt: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Certificate {
    nam: Name,
    dob: String,
    #[serde(default)]
    v: Vec<Vaccine>,
    #[serde(default)]
    r: Vec<Recovery>,
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

#[test]
fn decode_test() {
    // Taken from:
    // https://github.com/eu-digital-green-certificates/dgc-testdata/blob/main/IT/2DCode/raw/1.json
    // It is licensed under Apache-2.0 License.
    let vaccine_record_data = "HC1:6BFOXN%TS3DH0YOJ58S S-W5HDC *M0II5XHC9B5G2+$N IOP-IA%NFQGRJPC%OQHIZC4.OI1RM8ZA.A5:S9MKN4NN3F85QNCY0O%0VZ001HOC9JU0D0HT0HB2PL/IB*09B9LW4T*8+DCMH0LDK2%K:XFE70*LP$V25$0Q:J:4MO1P0%0L0HD+9E/HY+4J6TH48S%4K.GJ2PT3QY:GQ3TE2I+-CPHN6D7LLK*2HG%89UV-0LZ 2ZJJ524-LH/CJTK96L6SR9MU9DHGZ%P WUQRENS431T1XCNCF+47AY0-IFO0500TGPN8F5G.41Q2E4T8ALW.INSV$ 07UV5SR+BNQHNML7 /KD3TU 4V*CAT3ZGLQMI/XI%ZJNSBBXK2:UG%UJMI:TU+MMPZ5$/PMX19UE:-PSR3/$NU44CBE6DQ3D7B0FBOFX0DV2DGMB$YPF62I$60/F$Z2I6IFX21XNI-LM%3/DF/U6Z9FEOJVRLVW6K$UG+BKK57:1+D10%4K83F+1VWD1NE";
    let expected = Certificate {
        nam: Name {
            fn_: "Di Caprio".to_string(),
            fnt: "DI<CAPRIO".to_string(),
            gn: "Marilù Teresa".to_string(),
            gnt: "MARILU<TERESA".to_string(),
        },
        dob: "1977-06-16".to_string(),
        v: vec![Vaccine {
            ci: "01ITE7300E1AB2A84C719004F103DCB1F70A#6".to_string(),
            co: "IT".to_string(),
            dn: 2,
            dt: "2021-04-10".to_string(),
            is: "IT".to_string(),
            ma: "ORG-100030215".to_string(),
            mp: "EU/1/20/1528".to_string(),
            sd: 2,
            tg: "840539006".to_string(),
            vp: "1119349007".to_string(),
        }],
        r: vec![],
        ver: "1.0.0".to_string(),
    };

    let c1 = decode(vaccine_record_data.to_string()).unwrap();
    assert_eq!(c1, expected);
}
