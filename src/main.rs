use std::{env, fmt::Display};
use std::error::Error;
use std::time::Duration;
use teloxide::{prelude::*, types::ChatId};
use serialport::{available_ports, SerialPortType};
#[allow(dead_code)]

mod sensor;
use sensor::SensorCo2;
use tokio::time::sleep;

const LIMIT: i32 = 200;
const UART_LOOKING: u16 = 6790;


#[derive(Debug, Clone)]
struct NotFoundPortName;

impl Error for NotFoundPortName {}


impl Display for NotFoundPortName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Port is not found")
    }
}


fn get_uart_port_name() -> Result<String, Box<dyn Error>> {
    let ports = available_ports()?;
    for port in ports.into_iter() {
        match port.port_type {
            SerialPortType::UsbPort(port_info) => {
                if port_info.vid == UART_LOOKING {
                    return Ok(port.port_name)
                }
            },
            _ => {},
        }
    }
    Err(Box::new(NotFoundPortName))
}


#[tokio::main]
async fn main() {
    let chat_id: i64 = match env::var("CHAT_ID") {
        Ok(c) => c.parse::<i64>().unwrap(),
        Err(_) => 292317891,
    };

    let uart_port: String = get_uart_port_name().unwrap();

    let bot_token = match env::var("BOT_TOKEN"){
        Ok(v) => v,
        Err(_) => "5379849529:AAF72rf4rl0cCv4nzK-jNXT575niY_RXBDY".to_string(),
    };

    let mut device = match SensorCo2::new(uart_port) {
        Ok(device) => device,
        Err(err) => panic!("{}", err)
    };

    let bot = Bot::new(bot_token).auto_send();
    let chat = ChatId(chat_id);
    let mut last_level = 0;
    loop {
        match device.get_co2_value() {
            Ok(e) => {
                if (e - last_level).abs() > LIMIT {
                    let message = format!("Co2 level {}", e);
                    let rs = bot.send_message(chat, message).await;
                    match rs {
                        Ok(_) => {},
                        Err(e) => { println!("{}", e)}
                    }
                    last_level = e;
                }
            }
            Err(e) => {
                let message = format!("Err measure co2, {}", e);
                let rs = bot.send_message(chat, message).await;
                match rs {
                    Ok(_) => {},
                    Err(e) => { println!("{}", e)}
                }
            }
        }
        sleep(Duration::from_secs(5 * 60)).await;
    }
}
