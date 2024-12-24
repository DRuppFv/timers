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
    pub fn handle_events(self) -> Arc<Mutex<Quit>> {
        let quit: Arc<Mutex<Quit>> = Arc::new(Mutex::new(self));
        {
            let quit = Arc::clone(&quit);
            thread::spawn(move || {
                loop {
                    match event::read() {
                        Ok(Event::Key(key_event)) if key_event.kind == KeyEventKind::Press => {
                            match key_event.code {
                                KeyCode::Char('q') => {
                                    Quit::quit(&quit);
                                }
                                _ => {}
                            }
                        }
                        Err(_) => {} //HANDLING NEEDED TODO
                        _ => {}
                    };
                }
            });
        }

        return quit;
    }

    pub fn quit(self_arc: &Arc<Mutex<Self>>) {
        self_arc.lock().unwrap().bool = true;
    }
}
