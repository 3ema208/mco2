use serialport::{Error, ErrorKind, SerialPort};
use std::io::ErrorKind as ioErrKind;
use std::thread::sleep;
use std::time::Duration;

const START_COMMAND: u8 = 0x11;
const START_RESPONSE: u8 = 0x16;

 
#[derive(Clone)]
enum Command {
    ReadMeasure = 0x01,
    Calibration = 0x03,
    AbcParameterCheck = 0x0F,
    AbcParameterSet = 0x10,
}

struct SensorCo2 {
    serial: Box<dyn SerialPort>,
    buff: Vec<u8>,
}

fn calculate_crc(data: &[u8]) -> u8 {
    let sum = data.iter().sum::<u8>();
    255u8 - sum + 1
}

fn check_crc(data: &[u8], crc: &u8) -> bool {
    calculate_crc(data) == *crc
}

impl SensorCo2 {
    fn new(port: String) -> Result<Self, Error> {
        let port_builder = serialport::new(port, 9_600).timeout(Duration::from_secs(5));
        let port = port_builder.open()?;
        Ok(Self {
            serial: port,
            buff: vec![0; 8],
        })
    }

    fn pack_command(data: Vec<u8>) -> Vec<u8> {
        let mut r = vec![];
        r.push(START_COMMAND);
        r.push(data.len() as u8);
        r.extend(data.iter());
        r.push(calculate_crc(&r[..]));
        r
    }

    fn read_response(&mut self) -> Result<&[u8], Error> {
        let size = self.serial.read(&mut self.buff)?;
        if size == 0 {
            return Err(Error::new(ErrorKind::Io(ioErrKind::Other), "Can't read"));
        }
        let pack = &self.buff[..size - 1];
        let crc = self.buff.last().unwrap();
        if check_crc(pack, crc) {
            let params = pack[1] as usize;
            Ok(&self.buff[3..3 + params - 1])
        } else {
            Err(Error::new(ErrorKind::Io(ioErrKind::Other), "Wrong crc"))
        }
    }

    fn get_co2_value(&mut self) -> Result<i32, Error> {
        let command = SensorCo2::pack_command(vec![Command::ReadMeasure as u8]);
        self.serial.write_all(&command)?;
        let pack = self.read_response()?;
        let df1 = pack[0] as i32;
        let df2 = pack[1] as i32;
        Ok(df1 * 256 + df2)
    }
}

fn main() {
    let mut p = SensorCo2::new(String::from("/dev/tty.usbserial-14330")).unwrap();
    for _ in 0..10 {
        let co2_lvl = p.get_co2_value().unwrap();
        println!("Level co2: {:?}", co2_lvl);
        sleep(Duration::from_secs(120))
    }
}
