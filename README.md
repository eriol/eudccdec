# eudccdec - EU Digital COVID Certificate decoder

eudccdec is a decoder for EU Digital COVID Certificate (EUDCC), written in 
Rust and released under the GPLv3 license.

It ignores COSE signing and it extracts the EUDCC payload to show data about:
1. vaccination;
2. recovery;
3. tests;

## Installation

```
❯ cargo install --branch main --git https://noa.mornie.org/eriol/eudccdec
```

Note that `❯` is my shell prompt, you don't have to write it.

## Usage

In the following example `curl` and `zbarimg` are used, to install them on a
Debian based system use `sudo apt install curl zbar-tools`.

Example of a certificate with a vaccination entry:
```
❯ curl -sL https://github.com/eu-digital-green-certificates/dgc-testdata/raw/main/IT/png/1.png | \
  zbarimg --quiet --raw - | eudccdec
Certificate {
    ver: "1.0.0",
    nam: Name {
        fn_: "Di Caprio",
        fnt: "DI<CAPRIO",
        gn: "Marilù Teresa",
        gnt: "MARILU<TERESA",
    },
    dob: "1977-06-16",
    v: [
        VaccineRecord {
            tg: "840539006",
            vp: "1119349007",
            mp: "EU/1/20/1528",
            ma: "ORG-100030215",
            dn: 2,
            sd: 2,
            dt: "2021-04-10",
            co: "IT",
            is: "IT",
            ci: "01ITE7300E1AB2A84C719004F103DCB1F70A#6",
        },
    ],
    r: [],
    t: [],
}
```

Example of a certificate with a recovery entry:
```
❯ curl -sL https://github.com/eu-digital-green-certificates/dgc-testdata/raw/main/IT/png/2.png | \
  zbarimg --quiet --raw - | eudccdec
Certificate {
    ver: "1.0.0",
    nam: Name {
        fn_: "Di Caprio",
        fnt: "DI<CAPRIO",
        gn: "Marilù Teresa",
        gnt: "MARILU<TERESA",
    },
    dob: "1977-06-16",
    v: [],
    r: [
        RecoveryRecord {
            tg: "840539006",
            fr: "2021-05-02",
            co: "IT",
            is: "IT",
            df: "2021-05-04",
            du: "2021-10-31",
            ci: "01ITA65E2BD36C9E4900B0273D2E7C92EEB9#1",
        },
    ],
    t: [],
}
```

Example of a certificate with a test entry:
```
❯ curl -sL https://github.com/eu-digital-green-certificates/dgc-testdata/raw/main/IT/png/3.png | \
  zbarimg --quiet --raw - | eudccdec
Certificate {
    ver: "1.0.0",
    nam: Name {
        fn_: "Di Caprio",
        fnt: "DI<CAPRIO",
        gn: "Marilù Teresa",
        gnt: "MARILU<TERESA",
    },
    dob: "1977-06-16",
    v: [],
    r: [],
    t: [
        TestRecord {
            tg: "840539006",
            tt: "LP6464-4",
            nm: "Roche LightCycler qPCR",
            ma: "1232",
            sc: "2021-05-03T10:27:15Z",
            dr: "2021-05-11T12:27:15Z",
            tr: "260415000",
            tc: "Policlinico Umberto I",
            co: "IT",
            is: "IT",
            ci: "01IT053059F7676042D9BEE9F874C4901F9B#3",
        },
    ],
}
```
