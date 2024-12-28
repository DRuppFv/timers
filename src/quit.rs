use anyhow::anyhow;
use crossbeam_channel::Sender;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::{
    sync::{Arc, Mutex},
    thread,
};

#[derive(Debug, Default)]
pub struct Quit {
    pub bool: bool,
}

impl Quit {
    pub fn handle_events(self, sender: Sender<anyhow::Error>) -> Arc<Mutex<Self>> {
        let quit: Arc<Mutex<Self>> = Arc::new(Mutex::new(self));
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
                    Err(e) => {
                        if !sender.is_full() {
                            sender
                                .send(anyhow!("Failed to handle key event. Err: {}", e))
                                .unwrap();
                        }
                    }
                    _ => {}
                };
            });
        }

        quit
    }

    pub fn quit(self_arc: &Arc<Mutex<Self>>) {
        self_arc.lock().unwrap().bool = true;
    }
}
