use crate::quit::AppEvent;

use std::time::Duration;
use tokio::sync::mpsc::{UnboundedSender};
use tokio::time::interval;

#[derive(Debug)]
pub struct Counter {
    pub count: i32,
}

impl Counter {
    pub fn new(num: i32) -> Self {
        Self { count: num }
    }
    
    pub fn seconds(&self) -> i32 {
        self.count.abs() % 60
    }
    
    pub fn minutes(&self) -> i32 {
        self.count.abs() / 60 % 60
    }
    
    pub fn hours(&self) -> i32 {
        self.count.abs() / 3600
    }
}

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
