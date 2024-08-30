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
                            let decoded = String::from_utf8_lossy(data);
                            let decoded: String =
                                decoded.to_string().chars().skip(1).collect();
                            println!("Decoded msg: '{}'", decoded);
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
