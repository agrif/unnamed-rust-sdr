use super::signal::Signal;

use std::io::{Read, Result};
use std::net::{SocketAddr, IpAddr, Ipv4Addr, ToSocketAddrs};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

#[derive(Debug, Clone)]
pub struct RtlTcp {
    addr: Vec<SocketAddr>,
    rate: u32,
    frequency: u32,
    // others: AGC, tuner gain, RTL-AGC
}

impl RtlTcp {
    pub fn new() -> Self {
        RtlTcp {
            addr: vec![
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 1234),
            ],
            rate: 1800000,
            frequency: 100000000,
        }
    }

    pub fn address<A: ToSocketAddrs>(mut self, addr: A) -> Self {
        if let Ok(addrs) = addr.to_socket_addrs() {
            self.addr = addrs.collect();
        } else {
            self.addr = vec![];
        }
        self
    }

    pub fn rate(mut self, rate: u32) -> Self {
        self.rate = rate;
        self
    }

    pub fn frequency(mut self, frequency: u32) -> Self {
        self.frequency = frequency;
        self
    }

    pub fn listen(&self) -> Result<RtlTcpSignal> {
        let mut conn = RtlTcpConnection::connect(self.rate, &self.addr[..])?;
        conn.command(RtlTcpCommand::SetFrequency(self.frequency))?;
        Ok(conn.listen())
    }
}

#[derive(Debug)]
pub struct RtlTcpConnection {
    pub id: [u8; 12],
    stream: std::io::BufReader<std::net::TcpStream>,
    rate: u32,
}

#[derive(Debug)]
pub enum RtlTcpCommand {
    SetFrequency(u32), // Hz
    SetSampleRate(u32), // Hz
    SetTunerGainMode(u32), // manual != 0, agc == 0
    SetTunerGain(u32), // in 10ths of dB
    SetRtlAgc(u32), // AGC_ON != 0
}

impl RtlTcpConnection {
    pub fn connect<A: ToSocketAddrs>(rate: u32, addr: A) -> Result<Self> {
        let rawstream = std::net::TcpStream::connect(addr)?;
        let mut stream = std::io::BufReader::new(rawstream);
        let mut id = [0; 12];
        stream.read_exact(&mut id)?;
        let mut us = RtlTcpConnection {
            stream,
            id,
            rate,
        };
        us.command(RtlTcpCommand::SetSampleRate(rate))?;
        Ok(us)
    }

    pub fn command(&mut self, cmd: RtlTcpCommand) -> Result<()> {
        let (cmdi, arg) = match cmd {
            RtlTcpCommand::SetFrequency(a) => (0x01, a),
            RtlTcpCommand::SetSampleRate(a) => (0x02, a),
            RtlTcpCommand::SetTunerGainMode(a) => (0x03, a),
            RtlTcpCommand::SetTunerGain(a) => (0x04, a),
            RtlTcpCommand::SetRtlAgc(a) => (0x08, a),
        };

        self.stream.get_mut().write_u8(cmdi)?;
        self.stream.get_mut().write_u32::<BigEndian>(arg)?;

        if let RtlTcpCommand::SetSampleRate(rate) = cmd {
            // FIXME this will fail if rate is not
            // 225001 - 300000 Hz
            // 900001 - 3200000 Hz
            if !(225001 <= rate && rate <= 300000)
                && !(900001 <= rate && rate <= 3200000) {
                    panic!("bad sample rate for rtltcp: {:?}", rate);
                }
            self.rate = rate;
        }
        Ok(())
    }

    pub fn read(&mut self) -> Result<num::Complex<u8>> {
        let i = self.stream.read_u8()?;
        let q = self.stream.read_u8()?;
        Ok(num::Complex::new(i, q))
    }

    pub fn listen(self) -> RtlTcpSignal {
        RtlTcpSignal {
            rate: self.rate as f32,
            conn: self,
        }
    }
}

#[derive(Debug)]
pub struct RtlTcpSignal {
    conn: RtlTcpConnection,
    rate: f32,
}

impl Signal for RtlTcpSignal {
    type Sample = num::Complex<f32>;
    fn next(&mut self) -> Option<Self::Sample> {
        self.conn.read().ok()
            .map(|iq| num::Complex::new(
                (iq.re as f32 - 128.0) / 128.0,
                (iq.im as f32 - 128.0) / 128.0,
            ))
    }
    fn rate(&self) -> f32 {
        self.rate
    }
}
