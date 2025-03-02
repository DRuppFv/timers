use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::{
    sync::{self, atomic::AtomicBool, Arc},
    thread,
};
use tokio::sync::mpsc::UnboundedSender;

pub enum AppEvent {
    TickEvent,
    ErrorEvent,
}

#[derive(Debug, Default)]
pub struct Quit {
    pub bool: AtomicBool,
}

impl Quit {
    pub fn handle_events(self, sender: UnboundedSender<AppEvent>) -> Arc<Self> {
        let quit: Arc<Self> = Arc::new(self);
        {
            let quit = Arc::clone(&quit);
            thread::spawn(move || loop {
                match event::read() {
                    Ok(Event::Key(key_event)) if key_event.kind == KeyEventKind::Press => {
                        match key_event.code {
                            KeyCode::Char('q') => {
                                Self::quit(&quit);
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {
                        sender.send(AppEvent::ErrorEvent).unwrap();
                    }
                    _ => {}
                };
            });
        }

        quit
    }

    pub fn quit(self_arc: &Arc<Self>) {
        self_arc.bool.store(true, sync::atomic::Ordering::Relaxed);
    }
}
