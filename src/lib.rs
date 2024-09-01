use std::io::{self, Write};
use std::time::Duration;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    struct Status: u8 {
        const Retransmit = 1;
        const Continuation = 1 << 1;
        const ChangeType = 1 << 3;
    }
}

impl Default for Status {
    fn default() -> Status {
        Status::empty()
    }
}

impl From<Status> for u8 {
    fn from(val: Status) -> Self {
        val.bits()
    }
}

struct RawMessage<'a> {
    length: u8,
    opcode: OpCode,
    source: Source,
    status: Status,
    data: &'a [u8],
}

#[derive(Debug)]
enum DecodeError {
    InvalidChecksum,
    InvalidMessageLength,
}

#[derive(Debug)]
enum OpCode {
    Ack,
    Nack,
    DecodeData,
    Other(u8),
}

impl From<&u8> for OpCode {
    fn from(val: &u8) -> Self {
        match val {
            0xd0 => OpCode::Ack,
            0xd1 => OpCode::Nack,
            0xf3 => OpCode::DecodeData,
            _ => OpCode::Other(*val),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(val: OpCode) -> Self {
        match val {
            OpCode::Ack => 0xd0,
            OpCode::Nack => 0xd1,
            OpCode::DecodeData => 0xf3,
            OpCode::Other(val) => val,
        }
    }
}

#[derive(Debug)]
enum Source {
    Scanner,
    Host,
}

impl From<&u8> for Source {
    fn from(val: &u8) -> Self {
        match val {
            0x00 => Source::Scanner,
            0x04 => Source::Host,
            _ => unreachable!(),
        }
    }
}

impl From<Source> for u8 {
    fn from(val: Source) -> Self {
        match val {
            Source::Scanner => 0x00,
            Source::Host => 0x04,
        }
    }
}

#[repr(u8)]
#[derive(Debug)]
enum ContentType {
    Aztec = 0x2d,
    AztecRune = 0x2e,
    Bookland = 0x16,
    Chinese2of5 = 0x72,
    Codabar = 0x02,
    Code11 = 0x0c,
    Code128 = 0x03,
    Code16K = 0x12,
    Code32 = 0x20,
    Code39 = 0x01,
    Code39Ascii = 0x13,
    Code49 = 0x0d,
    Code93 = 0x07,
    // ... skipping composite codes ...
    Coupon = 0x17,
    CueCat = 0x38,
    Discrete2of5 = 0x04,
    DataMatrix = 0x1b,
    Dotcode = 0xc4,
    Ean13 = 0x0b,
    Ean13Plus2 = 0x4b,
    Ean13Plus5 = 0x8b,
    Ean8 = 0x0a,
    Ean8Plus2 = 0x4a,
    Ean8Plus5 = 0x8a,
    FrenchLottery = 0x2f,
    GridMatrix = 0xc8,
    Gs1_128 = 0x0f,
    Gs1DataBarExpanded = 0x32,
    Gs1DataBarLimited = 0x31,
    Gs1DataBar14 = 0x30,
    Gs1DataMatrix = 0xc1,
    Gs1Qr = 0xc2,
    HanXin = 0xb7,
    Iata = 0x05,
    Isbt128 = 0x19,
    Isbt128Concat = 0x21,
    Issn = 0x36,
    Interleaved2of5 = 0x06,
    Korean3of5 = 0x73, // Assuming here that '2 of 5' is a typo in the reference
    MacroMicroPdf = 0x9a,
    MacroPdf417 = 0x28,
    MacroQr = 0x29,
    Mailmark = 0xc3,
    Matrix2of5 = 0x39,
    Maxicode = 0x25,
    MicroPdf = 0x1a,
    MicroPdfCca = 0x1d,
    MicroQr = 0x2c,
    Msi = 0x0e,
    Multicode = 0xc6,
    Multipacket = 0x99,
    Nw7 = 0x18,
    OcrB = 0xa0,
    Pdf417 = 0x11,
    PlanetUs = 0x1f,
    PostalAus = 0x23,
    PostalNl = 0x24,
    PostalJp = 0x22,
    PostalUk = 0x27,
    PostbarCa = 0x26,
    PostnetUs = 0x1e,
    Qr = 0x1c,
    RfidRaw = 0xe0,
    RfidURI = 0xe1,
    RssExpandedCoupon = 0xb4,
    ScanletWebcode = 0x37,
    Signature = 0x69,
    Telepen = 0xca,
    Tlc39 = 0x5a,
    Trioptic = 0x15,
    UdiParsed = 0xcc,
    UpcA = 0x08,
    UpcAPlus2 = 0x48,
    UpcAPlus5 = 0x88,
    UpcE = 0x09,
    UpcEPlus2 = 0x49,
    UpcEPlus5 = 0x89,
    UpcE1 = 0x10,
    UpcE1Plus2 = 0x50,
    UpcE1Plus5 = 0x90,
    UkPlessy = 0xc7,
    FourStateUs = 0x34,
    FourStateUs4 = 0x35,
}

impl TryFrom<u8> for ContentType {
    type Error = &'static str;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        let output = match val {
            0x01 => Self::Code39,
            0x02 => Self::Codabar,
            0x03 => Self::Code128,
            0x04 => Self::Discrete2of5,
            0x05 => Self::Iata,
            0x06 => Self::Interleaved2of5,
            0x07 => Self::Code93,
            0x08 => Self::UpcA,
            0x09 => Self::UpcE,
            0x0a => Self::Ean8,
            0x0b => Self::Ean13,
            0x0c => Self::Code11,
            0x0d => Self::Code49,
            0x0e => Self::Msi,
            0x0f => Self::Gs1_128,
            0x10 => Self::UpcE1,
            0x11 => Self::Pdf417,
            0x12 => Self::Code16K,
            0x13 => Self::Code39Ascii,
            0x15 => Self::Trioptic,
            0x16 => Self::Bookland,
            0x17 => Self::Coupon,
            0x18 => Self::Nw7,
            0x19 => Self::Isbt128,
            0x1a => Self::MicroPdf,
            0x1b => Self::DataMatrix,
            0x1c => Self::Qr,
            0x1d => Self::MicroPdfCca,
            0x1e => Self::PostnetUs,
            0x1f => Self::PlanetUs,
            0x20 => Self::Code32,
            0x21 => Self::Isbt128Concat,
            0x22 => Self::PostalJp,
            0x23 => Self::PostalAus,
            0x24 => Self::PostalNl,
            0x25 => Self::Maxicode,
            0x26 => Self::PostbarCa,
            0x27 => Self::PostalUk,
            0x28 => Self::MacroPdf417,
            0x29 => Self::MacroQr,
            0x2c => Self::MicroQr,
            0x2d => Self::Aztec,
            0x2e => Self::AztecRune,
            0x2f => Self::FrenchLottery,
            0x30 => Self::Gs1DataBar14,
            0x31 => Self::Gs1DataBarLimited,
            0x32 => Self::Gs1DataBarExpanded,
            0x34 => Self::FourStateUs,
            0x35 => Self::FourStateUs4,
            0x36 => Self::Issn, // Not listed in reference
            0x37 => Self::ScanletWebcode,
            0x38 => Self::CueCat,
            0x48 => Self::UpcAPlus2,
            0x49 => Self::UpcEPlus2,
            0x4a => Self::Ean8Plus2,
            0x4b => Self::Ean13Plus2,
            0x50 => Self::UpcE1Plus2,
            // ... skipped composite codes as above ...
            0x5a => Self::Tlc39,
            0x69 => Self::Signature,
            0x71 => Self::Matrix2of5,
            0x72 => Self::Chinese2of5,
            0x73 => Self::Korean3of5,
            0x88 => Self::UpcAPlus5,
            0x89 => Self::UpcEPlus5,
            0x8a => Self::Ean8Plus5,
            0x8b => Self::Ean13Plus5,
            0x90 => Self::UpcE1Plus5,
            0x99 => Self::Multipacket,
            0x9a => Self::MacroMicroPdf,
            0xa0 => Self::OcrB,
            0xb4 => Self::RssExpandedCoupon,
            0xb7 => Self::HanXin,
            0xc1 => Self::Gs1DataMatrix,
            0xc2 => Self::Gs1Qr,
            0xc3 => Self::Mailmark,
            0xc4 => Self::Dotcode,
            0xc6 => Self::Multicode,
            0xc7 => Self::UkPlessy,
            0xc8 => Self::GridMatrix,
            0xca => Self::Telepen,
            0xcc => Self::UdiParsed,
            0xe0 => Self::RfidRaw,
            0xe1 => Self::RfidURI,
            _ => return Err("Unknown content type"),
        };

        Ok(output)
    }
}

fn calc_checksum(size: u8, payload: &[u8]) -> u16 {
    size as u16 + payload.iter().cloned().map(u16::from).sum::<u16>()
}

fn decode(message: &[u8]) -> Result<RawMessage, DecodeError> {
    let [length, payload @ .., checksum1, checksum2] = message else {
        return Err(DecodeError::InvalidMessageLength);
    };

    // Integrity check
    let checksum = -i16::from_be_bytes([*checksum1, *checksum2]) as u16;
    let sum: u16 = calc_checksum(*length, payload);

    if sum != checksum {
        return Err(DecodeError::InvalidChecksum);
    }

    let [opcode, source, status, data @ ..] = payload else {
        return Err(DecodeError::InvalidMessageLength);
    };

    Ok(RawMessage {
        length: *length,
        opcode: opcode.into(),
        source: source.into(),
        // Truncation ignores unknown bits
        status: Status::from_bits_truncate(*status),
        data,
    })
}

fn wrap(data: Vec<u8>) -> Vec<u8> {
    // Size counts the size itself
    let size = data.len() as u8 + 1;
    // Checksum includes the size
    let checksum = calc_checksum(size, &data);

    let mut output = vec![size];
    output.extend(data);
    output.extend((-(checksum as i16)).to_be_bytes());

    output
}

pub async fn run(port_name: &str, baud_rate: u32) {
    let port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(10))
        .open();

    let mut port = match port {
        Ok(port) => port,
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    };

    println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);

    let mut serial_buf: Vec<u8> = vec![0; 1000];
    loop {
        match port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                // TODO: Check length of t
                // TODO: Investigate #[repr(C, packed)] to unpack into struct
                let message = &serial_buf[..t];
                let response = decode(message);

                match response {
                    Ok(RawMessage {
                        length,
                        opcode,
                        source,
                        status,
                        data,
                    }) => {
                        let ack = wrap(vec![
                            OpCode::Ack.into(),
                            Source::Host.into(),
                            Status::default().into(),
                        ]);
                        port.write(&ack).unwrap();

                        println!("Length: {length}");
                        println!("Opcode: {opcode:?}");
                        println!("Source: {source:?}");
                        println!("Status: {status:?}");

                        if let OpCode::DecodeData = opcode {
                            if let [content_type, content @ ..] = data {
                                match <ContentType as TryFrom<u8>>::try_from(
                                    *content_type,
                                ) {
                                    Ok(content_type) => {
                                        println!("Type: '{:?}'", content_type);
                                    }
                                    Err(_) => {
                                        println!(
                                            "Unknown type: '{:#04x}'",
                                            content_type
                                        );
                                    }
                                }

                                let decoded = String::from_utf8_lossy(content);
                                println!("Decoded msg: '{}'", decoded);
                            } else {
                                println!("Invalid DecodeData");
                            };
                        }
                    }
                    Err(decode_error) => {
                        println!("Error decoding data: {decode_error:?}");
                    }
                };
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
