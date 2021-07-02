# eudccdec

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

## Usage

In the following example `curl` and `zbarimg` are used, to install them on a
Debian based system use `sudo apt install curl zbar-tools`.

```
❯ curl -sL https://github.com/eu-digital-green-certificates/dgc-testdata/blob/main/IT/png/1.png | \
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
