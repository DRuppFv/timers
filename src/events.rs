use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::thread;
use tokio::sync::mpsc::UnboundedSender;

pub enum AppEvent {
    Tick,
    Quit,
    Error,
}

pub fn handle_crossterm_events(sender: UnboundedSender<AppEvent>) {
    thread::spawn(move || loop {
        match event::read() {
            Ok(Event::Key(key_event)) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => sender.send(AppEvent::Quit).unwrap(),
                    _ => {}
                }
            }
            Err(_) => {
                sender.send(AppEvent::Error).unwrap();
            }
            _ => {}
        };
    });
}
