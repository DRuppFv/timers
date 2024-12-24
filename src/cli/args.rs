use anyhow::anyhow;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about)]
pub struct Args {
    ///Time in the format __h__m__s, the order doesn't matter.
    #[arg(default_value = "15m00s")]
    time: Option<String>,
}

impl Args {
    pub fn handle_command(&mut self, counter: &mut crate::counter::Counter) -> anyhow::Result<()> {
        if let Some(time) = self.time.as_deref() {
            //
            let mut seconds: i32 = 0;

            let mut current_numeric = String::new();
            let mut order_str = Vec::new();

            for ch in time.chars() {
                if (ch == 'h' || ch == 'm' || ch == 's') && !order_str.contains(&ch) {
                    order_str.push(ch);
                    let unit = current_numeric.parse::<i32>()?;
                    seconds = match ch {
                        'h' => seconds + unit * 3600,
                        'm' => seconds + unit * 60,
                        's' => seconds + unit,
                        _ => unreachable!(),
                    };
                    current_numeric.clear();
                } else {
                    current_numeric.push(ch);
                }
            }

            if !current_numeric.is_empty() {
                return Err(anyhow!("Command format not recognized."));
            };

            counter.count = seconds;
            Ok(())
            //
        } else {
            return Err(anyhow!("No duration argument provided."));
        }
    }
}
