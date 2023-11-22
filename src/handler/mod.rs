use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::RwLock;
use std::thread;

use crate::receiver::BroadcastReceiver;
use crate::sender::BroadcastSender;

pub struct BroadcastHandler<T: Clone> {
    channels: RwLock<Vec<InternalChannel<T>>>,
    individual_channels_settings: IndividualSettings,
    settings: BroadcastSettings<T>,
}

struct InternalChannel<T: Clone> {
    sender: Sender<BroadcastMessage<T>>,
    receiver: Receiver<BroadcastMessage<T>>,
    setting: IndividualSettings,
}

impl<T: Clone> BroadcastHandler<T> {
    pub fn new() -> BroadcastHandler<T> {
        Self {
            channels: Default::default(),
            individual_channels_settings: Default::default(),
            settings: Default::default(),
        }
    }

    pub fn new_channel_settings(&mut self) -> &mut IndividualSettings {
        &mut self.individual_channels_settings
    }

    pub fn handler_settings(&mut self) -> &mut BroadcastSettings<T> {
        &mut self.settings
    }

    pub fn generate_channel(&self) -> Result<(BroadcastSender<T>, BroadcastReceiver<T>), ()> {
        let mut lock = self.channels.write().map_err(|_| ())?;
        let broadcast_channels = &mut *lock;
        let (broadcast_sender, individual_receiver) = channel::<BroadcastMessage<T>>();
        let (individual_sender, broadcast_receiver) = channel::<BroadcastMessage<T>>();
        let internal_chanel = InternalChannel {
            sender: broadcast_sender,
            receiver: broadcast_receiver,
            setting: self.individual_channels_settings.clone()
        };
        broadcast_channels.push(internal_chanel);
        Ok((BroadcastSender::new(individual_sender), BroadcastReceiver::new(individual_receiver)))
    }

    pub fn handle_once_cicle(&self) -> Result<(), ()> {
        let mut lock = self.channels.write().map_err(|_| ())?;
        let broadcast_channels = &mut *lock;
        let mut channel_index = 0;
        while channel_index < broadcast_channels.len() {
            loop {
                let recv = broadcast_channels[channel_index].receiver.try_recv();
                if recv.is_err() {
                    if recv.err().unwrap().eq(&TryRecvError::Disconnected){
                        broadcast_channels.remove(channel_index);
                    } else {
                        channel_index += 1
                    }
                    break;
                }
                match recv.ok().unwrap() {
                    BroadcastMessage::Value(value) => {
                        let skip_self_add = if broadcast_channels[channel_index].setting.send_to_self { 0 } else { 1 };
                        let before_sender = &broadcast_channels[0..channel_index];
                        let after_sender = &broadcast_channels[channel_index + skip_self_add..broadcast_channels.len()];
                        let mut remaining_broadcasts = before_sender.len() + after_sender.len();
                        if remaining_broadcasts == 0 {
                            continue;
                        }
                        let mut broad_casts_iter = before_sender.iter().chain(after_sender.iter());
                        while remaining_broadcasts > 1 {
                            remaining_broadcasts -= 1;
                            broad_casts_iter.next().unwrap().sender.send(BroadcastMessage::Value(value.clone()));
                        }
                        remaining_broadcasts -= 1;
                        broad_casts_iter.next().unwrap().sender.send(BroadcastMessage::Value(value));
                        if !self.settings.exit_on_disconnected_channels{ continue; }
                        channel_index += 1;
                        break;
                    }
                    BroadcastMessage::ChangeSendToSelf(new_settings) => {
                        broadcast_channels[channel_index].setting.send_to_self = new_settings
                    }
                };
            }
        }
        Ok(())
    }

    pub fn handle_block(&self) {
        loop {
            let is_error = self.handle_once_cicle().is_err();
            let is_empty = self.settings.exit_on_disconnected_channels && match self.channels.read() {
                Ok(channels_guard) => { channels_guard.is_empty() }
                Err(_) => { true }
            };
            let is_custom_disconnect = (self.settings.disconnect_reason)(self);
            if is_error || is_empty || is_custom_disconnect { break; }
        }
    }
}

pub enum BroadcastMessage<T> {
    Value(T),
    ChangeSendToSelf(bool),
}

impl<T> BroadcastMessage<T> {
    pub(crate) fn force_value(self) -> T {
        match self {
            BroadcastMessage::Value(value) => { value }
            BroadcastMessage::ChangeSendToSelf(_) => { panic!("Broadcast Receiver received wrong value") }
        }
    }
}

#[derive(Clone)]
pub struct IndividualSettings {
    pub send_to_self: bool,
}

impl Default for IndividualSettings {
    fn default() -> Self {
        Self {
            send_to_self: false,
        }
    }
}

pub struct BroadcastSettings<T: Clone> {
    pub exit_on_disconnected_channels: bool,
    pub process_one_message_per_cicle: bool,
    pub disconnect_reason: fn(&BroadcastHandler<T>) -> bool,
}

impl<T: Clone> Default for BroadcastSettings<T> {
    fn default() -> Self {
        Self {
            exit_on_disconnected_channels: true,
            process_one_message_per_cicle: true,
            disconnect_reason: |_| false,
        }
    }
}
