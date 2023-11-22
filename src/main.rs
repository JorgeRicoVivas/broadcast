extern crate core;

use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const NUM_OF_THREADS: usize = 3;


fn main() {
    let mut broadcast_handler = handler::BroadcastHandler::new();
    broadcast_handler.new_channel_settings().send_to_self=false;
    for i in 0..NUM_OF_THREADS {
        let (sender,receiver) = broadcast_handler.generate_channel().unwrap();
        thread::spawn(move || {
            let numbers = (0..=(i+1)*3).into_iter();
            numbers.for_each(|number|{
                println!("[Thread {}] Sending value {}", i+1, number);
                sender.send(number);
                println!("[Thread {}] Others have sent values {:?}", i+1,  receiver.try_iter().collect::<Vec<_>>());
                sleep(Duration::from_millis(500));

            });
            println!("[Thread {}] My task is finished", i+1);
        });
    }
    broadcast_handler.handle_block();
}

pub mod handler;
pub mod sender;
pub mod receiver;