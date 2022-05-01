use std::time::Duration;
use std::env;
use teloxide::{
    prelude::*,
    types::{ChatId},
};

mod sensor;
use sensor::SensorCo2;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let chat_id: i64 = env::var("CHAT_ID").unwrap().parse::<i64>().unwrap();
    let uart_port = env::var("UART_PORT").unwrap();
    let bot_token = env::var("BOT_TOKEN").unwrap();
    
    
    pretty_env_logger::init();
    let mut device = SensorCo2::new(uart_port).unwrap();
    log::info!("Starting throw dice bot...");
    let bot = Bot::new(bot_token).auto_send();
        
    let chat = ChatId(chat_id);
    let last_level = 0;
    loop {
        match device.get_co2_value() {
            Ok(e) => {
                if e - last_level > 200 {
                    let message = format!("Co2 level {}", e);
                    bot.send_message(chat, message).await.unwrap();
                }
            }
            Err(e) => {
                let message = format!("Err measure co2, {}", e);
                bot.send_message(chat, message).await.unwrap();
            }
        }
        sleep(Duration::from_secs(120)).await;
    }
}
// })
