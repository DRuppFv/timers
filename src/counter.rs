use crate::quit::AppEvent;

use std::time::Duration;
use tokio::sync::mpsc::{UnboundedSender};
use tokio::time::interval;

#[derive(Debug, Default)]
pub struct Counter {
    pub count: i32,
}

impl Counter {}

pub fn start_ticking(sender: UnboundedSender<AppEvent>) {
    let mut interval = interval(Duration::from_secs(1));
    
    tokio::spawn(async move {
        interval.tick().await;
        loop {
            interval.tick().await;
            sender.send(AppEvent::TickEvent).unwrap();
        }
    });
}
