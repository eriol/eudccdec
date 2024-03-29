use std::collections::HashMap;
use std::fmt;
use std::io::Read;

use anyhow::{bail, Result};
use base45;
use ciborium::{de::from_reader, value::Value};
use flate2::read::ZlibDecoder;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::Deserialize;

const CLAIM_KEY_DCCV1: usize = 1; // EU Digital Covid Certificate v1
const CLAIM_KEY_EXPIRETION_TIME: i16 = 4;
const CLAIM_KEY_HCERT: i16 = -260;
const CLAIM_KEY_ISSUED_AT: i16 = 6;
const CLAIM_KEY_ISSUER: i16 = 1;
const COSE_SIGN1_TAG: u64 = 18;
const HC1_FIELD: &str = "HC1:";
const PAYLOAD_POSITION: usize = 2;

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct VaccineRecord {
    tg: String,
    vp: String,
    mp: String,
    ma: String,
    dn: i32,
    sd: i32,
    dt: String,
    co: String,
    is: String,
    ci: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct RecoveryRecord {
    tg: String,
    fr: String,
    co: String,
    is: String,
    df: String,
    du: String,
    ci: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct TestRecord {
    tg: String,
    tt: String,
    #[serde(default)]
    nm: String,
    #[serde(default)]
    ma: String,
    sc: String,
    #[serde(default)]
    dr: String,
    tr: String,
    tc: String,
    co: String,
    is: String,
    ci: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Name {
    #[serde(rename = "fn")]
    fn_: String,
    fnt: String,
    gn: String,
    gnt: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Certificate {
    ver: String,
    nam: Name,
    dob: String,
    #[serde(default)]
    v: Vec<VaccineRecord>,
    #[serde(default)]
    r: Vec<RecoveryRecord>,
    #[serde(default)]
    t: Vec<TestRecord>,
}

#[derive(Debug, PartialEq)]
pub struct Payload {
    pub expires_at: u64,
    pub issued_at: u64,
    pub issuer: String,
    pub certs: HashMap<usize, Certificate>,
}

impl<'de> Deserialize<'de> for Payload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PayloadVisitor;

        impl<'de> Visitor<'de> for PayloadVisitor {
            type Value = Payload;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Payload")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Payload, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut issued_at = None;
                let mut issuer = None;
                let mut expires_at = None;
                let mut certs = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        CLAIM_KEY_ISSUER => {
                            if issuer.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "issuer",
                                ));
                            }
                            issuer = Some(map.next_value()?);
                        }
                        CLAIM_KEY_ISSUED_AT => {
                            if issued_at.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "issued_at",
                                ));
                            }
                            issued_at = Some(map.next_value()?);
                        }
                        CLAIM_KEY_EXPIRETION_TIME => {
                            if expires_at.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "expires_at",
                                ));
                            }
                            expires_at = Some(map.next_value()?);
                        }
                        CLAIM_KEY_HCERT => {
                            if certs.is_some() {
                                return Err(de::Error::duplicate_field(
                                    "certs",
                                ));
                            }
                            certs = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore the rest.
                        }
                    }
                }
                let issuer =
                    issuer.ok_or_else(|| de::Error::missing_field("issuer"))?;
                let issued_at = issued_at
                    .ok_or_else(|| de::Error::missing_field("issued_at"))?;
                let expires_at = expires_at
                    .ok_or_else(|| de::Error::missing_field("expire_at"))?;
                let certs =
                    certs.ok_or_else(|| de::Error::missing_field("certs"))?;
                Ok(Payload {
                    issuer,
                    issued_at,
                    expires_at,
                    certs,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["issuer", "issued_at"];
        deserializer.deserialize_struct("Payload", FIELDS, PayloadVisitor)
    }
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

    if let Value::Tag(COSE_SIGN1_TAG, content) =
        ciborium::de::from_reader(&cbor_data[..])?
    {
        if let Value::Array(arr) = *content {
            // We have 4 part of a CBOR Web Token:
            // 1. protected header;
            // 2. unprotected header;
            // 3. payload;
            // 4. signature.
            if let Value::Bytes(p) = &arr[PAYLOAD_POSITION] {
                let p: Payload = from_reader(&p[..])?;
                let cert = p.certs[&CLAIM_KEY_DCCV1].clone();
                return Ok(cert);
            }
        }
    } else {
        bail!("Not a COSE Single Signer Data Object Tag!")
    }

    bail!("Can't decode the EU Digital COVID Certificate payload!");
}

#[test]
fn decode_vaccination_test() {
    // Taken from:
    // https://github.com/eu-digital-green-certificates/dgc-testdata/blob/main/IT/2DCode/raw/1.json
    // It is licensed under Apache-2.0 License.
    let vaccination_data = "HC1:6BFOXN%TS3DH0YOJ58S S-W5HDC *M0II5XHC9B5G2+$N IOP-IA%NFQGRJPC%OQHIZC4.OI1RM8ZA.A5:S9MKN4NN3F85QNCY0O%0VZ001HOC9JU0D0HT0HB2PL/IB*09B9LW4T*8+DCMH0LDK2%K:XFE70*LP$V25$0Q:J:4MO1P0%0L0HD+9E/HY+4J6TH48S%4K.GJ2PT3QY:GQ3TE2I+-CPHN6D7LLK*2HG%89UV-0LZ 2ZJJ524-LH/CJTK96L6SR9MU9DHGZ%P WUQRENS431T1XCNCF+47AY0-IFO0500TGPN8F5G.41Q2E4T8ALW.INSV$ 07UV5SR+BNQHNML7 /KD3TU 4V*CAT3ZGLQMI/XI%ZJNSBBXK2:UG%UJMI:TU+MMPZ5$/PMX19UE:-PSR3/$NU44CBE6DQ3D7B0FBOFX0DV2DGMB$YPF62I$60/F$Z2I6IFX21XNI-LM%3/DF/U6Z9FEOJVRLVW6K$UG+BKK57:1+D10%4K83F+1VWD1NE";
    let expected = Certificate {
        ver: "1.0.0".to_string(),
        nam: Name {
            fn_: "Di Caprio".to_string(),
            fnt: "DI<CAPRIO".to_string(),
            gn: "Marilù Teresa".to_string(),
            gnt: "MARILU<TERESA".to_string(),
        },
        dob: "1977-06-16".to_string(),
        v: vec![VaccineRecord {
            tg: "840539006".to_string(),
            vp: "1119349007".to_string(),
            mp: "EU/1/20/1528".to_string(),
            ma: "ORG-100030215".to_string(),
            dn: 2,
            sd: 2,
            dt: "2021-04-10".to_string(),
            co: "IT".to_string(),
            is: "IT".to_string(),
            ci: "01ITE7300E1AB2A84C719004F103DCB1F70A#6".to_string(),
        }],
        r: vec![],
        t: vec![],
    };

    let c = decode(vaccination_data.to_string()).unwrap();
    assert_eq!(c, expected);
}

#[test]
fn decode_recovery_test() {
    // Taken from:
    // https://github.com/eu-digital-green-certificates/dgc-testdata/blob/main/IT/2DCode/raw/2.json
    // It is licensed under Apache-2.0 License.
    let recovery_data = "HC1:6BFOXN%TS3DH0YOJ58S S-W5HDC *MEB2B2JJ59J-9BC6:X9NECX0AKQC:3DCV4*XUA2P-FHT-H4SI/J9WVHWVH+ZEOV1J$HNTICZUBOM*LP$V25$0Q:J40IA3L/*84-5%:C92JN*4CY0*%9F/8J2P4.818T+:IX3M3.96RPVD9J-OZT1-NT0 2$$0$2PZX69B9VCDHI2/T9TU1BPIJKH/T7B-S-*O/Y41FD+X49+5Z-6%.HDD8R6W1FDJGJSFJ/4Q:T0.KJTNP8EFULNC:HA0K5HKRB4TD85LOLF92GF.3O.Z8CC7-2FQYG$%21 2O*4R60NM8JI0EUGP$I/XK$M8ZQE6YB9M66P8N31I.ROSK%IA1Q2N53Q-OQ2VC6E26T11ROSNK5W-*H+MJ%0RGZVGWNURI75RBSQSHLH1JG*CMH2.-S$7VX6N*Z1881J7G.F9I+SV06F+1M*93%D";
    let expected = Certificate {
        ver: "1.0.0".to_string(),
        nam: Name {
            fn_: "Di Caprio".to_string(),
            fnt: "DI<CAPRIO".to_string(),
            gn: "Marilù Teresa".to_string(),
            gnt: "MARILU<TERESA".to_string(),
        },
        dob: "1977-06-16".to_string(),
        v: vec![],
        r: vec![RecoveryRecord {
            tg: "840539006".to_string(),
            fr: "2021-05-02".to_string(),
            co: "IT".to_string(),
            is: "IT".to_string(),
            df: "2021-05-04".to_string(),
            du: "2021-10-31".to_string(),
            ci: "01ITA65E2BD36C9E4900B0273D2E7C92EEB9#1".to_string(),
        }],
        t: vec![],
    };

    let c = decode(recovery_data.to_string()).unwrap();
    assert_eq!(c, expected);
}

#[test]
fn decode_test_test() {
    // Taken from:
    // https://github.com/eu-digital-green-certificates/dgc-testdata/blob/main/IT/2DCode/raw/3.json
    // It is licensed under Apache-2.0 License.
    let test_data = "HC1:6BFOXN%TS3DH0YOJ58S S-W5HDC *M0IIE 1C9B5G2+$NP-OP-IA%N%QHRJPC%OQHIZC4.OI:OIG/Q80P2W4VZ0K1H$$0CNN62PK.G +AG5T01HJCAMKNAB5S.8%*8Z95%9EMP8N22MM42WFCD9C2AKIJKIJM1MQIAY.D-7A4KE0PLV1ARKF.GH5$C4-9GGIUEC0QE1JAF.714NTPINRQ3.VR+P0$J2*N$*SB-G9+RT*QFNI2X02%KYZPQV6YP8412HOA-I0+M9GPEGPEMH0SJ4OM9*1B+M96K1HK2YJ2PI0P:65:41ZSW$P*CM-NT0 2$88L/II 05B9.Z8T*8Y1VM:KCY07LPMIH-O9XZQ4H9IZBP%D2U3+KGP2W2UQNG6-E6+WJTK1%J6/UI2YUELE+W35T7+H8NH8DRG+PG.UIZ$U%UF*QHOOENBU621TW5XW5HS9+I010H% 0R%0ZD5CC9T0HP8TCNNI:CQ:G172DX8FZV3U9W-HNPPQ N2KV 2VHDHO:2XAV:FB+18DRR%%VQ F60LF6K 38GK8LGG4U7UP6*S4QBR-F97FRONPKZS+P9$5W1CAV37KD48ERCRH";
    let expected = Certificate {
        ver: "1.0.0".to_string(),
        nam: Name {
            fn_: "Di Caprio".to_string(),
            fnt: "DI<CAPRIO".to_string(),
            gn: "Marilù Teresa".to_string(),
            gnt: "MARILU<TERESA".to_string(),
        },
        dob: "1977-06-16".to_string(),
        v: vec![],
        r: vec![],
        t: vec![TestRecord {
            tg: "840539006".to_string(),
            tt: "LP6464-4".to_string(),
            nm: "Roche LightCycler qPCR".to_string(),
            ma: "1232".to_string(),
            sc: "2021-05-03T10:27:15Z".to_string(),
            dr: "2021-05-11T12:27:15Z".to_string(),
            tr: "260415000".to_string(),
            tc: "Policlinico Umberto I".to_string(),
            co: "IT".to_string(),
            is: "IT".to_string(),
            ci: "01IT053059F7676042D9BEE9F874C4901F9B#3".to_string(),
        }],
    };

    let c = decode(test_data.to_string()).unwrap();
    assert_eq!(c, expected);

    // Taken from:
    // https://github.com/eu-digital-green-certificates/dgc-testdata/blob/main/IT/2DCode/raw/4.json
    // It is licensed under Apache-2.0 License.
    let test_data = "HC1:6BFOXN%TS3DH0YOJ58S S-W5HDC *M0II*%6C9B5G2+$NEJPP-IA%NGRIRJPC%OQHIZC4.OI:OIG/Q80P2W4VZ0K1H$$05QN*Y0K.G +AG5T01HJCAMKN$71Z95Z11VTO.L8YBJ-B93:GQBGZHHBIH5C99.B4DBF:F0.8ELG:.CC-8LQECKEBLDSH8XAG.6A-JE:GQA KX-SZDG0$JO+SW*PR+PHXF8IQV$K%OKOUFBBQR-S3D1PI0/7Q.H0807-L9CL62/2JJ11K2919GI1X1DDM8RMA0/41:6Z.2:NC-%CN$KJLCLF9+FJE 4Y3LL/II 05B9.Z8M+8:Y001HCY0R%0IGF5JNCPIGSUNG6YS75XJ/J0/V7.UI$RU8ZB.W2FI28LHUZUYZQNI9Y FQQGQ$FP DDVBDVBBX33UQLTU8L20H6/*12SADB9:G9J+9Y 5LJA8JF8JFHJP7NVDEBK3JQ7TI 05QNT+CCZ1ZA2I+T*R9XZ6/:COTJCURIF8CZPCJ4EF5LU5I-Q:.N$P9DX5NAM*PJYD3L2V0GBG.JL4LESU72S1CM%5OC%VSTJ8NC1TGO:QS02V505GJUTH";
    let expected = Certificate {
        ver: "1.0.0".to_string(),
        nam: Name {
            fn_: "Di Caprio".to_string(),
            fnt: "DI<CAPRIO".to_string(),
            gn: "Marilù Teresa".to_string(),
            gnt: "MARILU<TERESA".to_string(),
        },
        dob: "1977-06-16".to_string(),
        v: vec![],
        r: vec![],
        t: vec![TestRecord {
            tg: "840539006".to_string(),
            tt: "LP6464-4".to_string(),
            nm: "Roche LightCycler qPCR".to_string(),
            ma: "".to_string(),
            sc: "2021-05-10T10:27:15Z".to_string(),
            dr: "2021-05-11T12:27:15Z".to_string(),
            tr: "260415000".to_string(),
            tc: "Policlinico Umberto I".to_string(),
            co: "IT".to_string(),
            is: "IT".to_string(),
            ci: "01IT0BFC9866D3854EAC82C21654B6F6DE32#1".to_string(),
        }],
    };

    let c = decode(test_data.to_string()).unwrap();
    assert_eq!(c, expected);
}
