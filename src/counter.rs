use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Debug, Default)]
pub struct Counter {
    pub count: i32,
}

impl Counter {
    pub fn start_counting(self) -> Arc<Mutex<Self>> {
        let contador = Arc::new(Mutex::new(self));
        {
            let contador = Arc::clone(&contador);
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));
                let mut locked_data = contador.lock().unwrap();
                locked_data.count -= 1;
            });
        }

        contador
    }
}
