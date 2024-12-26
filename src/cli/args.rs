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
        if let Some(time_arg) = self.time.as_deref() {
            //
            let seconds: i32 = Args::parse_time(time_arg)?;

            counter.count = seconds;
            Ok(())
            //
        } else {
            return Err(anyhow!("No duration argument provided."));
        }
    }

    fn parse_time(time: &str) -> Result<i32, anyhow::Error> {
        let mut seconds: i32 = 0;

        if time.contains(':') {
            let split_time = time.split(':');
            for (i, s) in split_time.rev().enumerate() {
                let units = s.parse::<i32>()?;
                seconds = match i {
                    0 => seconds + units,
                    1 => seconds + units * 60,
                    _ => seconds + units * 3600,
                }
            }
        } else {
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
        }

        Ok(seconds)
    }
}
