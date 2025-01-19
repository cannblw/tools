use std::{
    error::Error,
    process::{Command, Output},
    str, thread,
    time::Duration,
};

// We could make this a CLI setting
/// Maximum maximum_interval in seconds to run the battery check
/// This will usually happen when your battery level is 50%
const MAX_INTERVAL_IN_SECONDS: i32 = 20 * 60;

fn get_battery_level() -> Result<i32, Box<dyn Error>> {
    let output = Command::new("sh")
        .arg("-c")
        .arg("pmset -g batt | grep -Eo '\\d+%' | cut -d% -f1")
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Command failed with status {}",
            output.status.code().unwrap_or(-1)
        )
        .into());
    }

    let output_str = str::from_utf8(&output.stdout)?;

    output_str.trim().parse::<i32>().map_err(|e| e.into())
}

fn display_alert(title: &str, message: &str) -> Result<Output, std::io::Error> {
    Command::new("osascript")
        .arg("-e")
        .arg(format!(
            r#"display dialog "{}" with title "{}" buttons {{"OK"}} default button "OK""#,
            message, title
        ))
        .output()
}

/// Calculate how much time to sleep depending on your current battery level
/// This is calculated using a simple parabolic function (y=ax^2+bx+c). The sleep time is the highest
/// when your battery level is 50%, and it's always 1 minute when you battery level is
/// 20% and 80%. The interval varies more aggresively when you get closer to 20% and 80%,
/// and it's relatively stable when you're close to 50%
fn get_sleep_seconds(current_battery_level: i32, maximum_interval_in_seconds: i32) -> i32 {
    let maximum_interval_in_seconds = maximum_interval_in_seconds as f32;

    let a = (60.0 - maximum_interval_in_seconds) / 900.0;
    let b = (maximum_interval_in_seconds - 60.0) / 9.0;
    let c = 4.0 / 9.0 * (375.0 - 4.0 * maximum_interval_in_seconds);

    let x = current_battery_level as f32;
    let y = a * x.powi(2) + b * x + c;

    y.round() as i32
}

/// Display the alert if the battery is at dangerous levels.
/// The alert is only fired once, and will then wait until the battery gets to safe levels
/// before attempting to trigger it again. We don't want the alert to be triggered non-stop
fn display_alert_if_needed(battery_level: i32, is_alert_allowed: &mut bool) {
    if *is_alert_allowed && battery_level <= 20 {
        display_alert(
            "Battery Low",
            &format!("Battery is at {}%. Please charge it.", battery_level),
        )
        .expect("Error displaying alert");

        *is_alert_allowed = false;
    } else if *is_alert_allowed && battery_level >= 80 {
        display_alert(
            "Battery High",
            &format!("Battery is at {}%. Consider unplugging.", battery_level),
        )
        .expect("Error displaying alert");

        *is_alert_allowed = false;
    }
}

fn main() {
    println!("== MacBook battery 20%-80% running ==");
    let mut is_alert_allowed = true;

    loop {
        let battery_level = match get_battery_level() {
            Ok(level) => level,
            Err(err) => {
                eprintln!(
                    "Error getting battery level. Skipping check. Error: {}",
                    err
                );
                display_alert(
                    "Error in MacBook Battery 20%-80% script",
                    "Error reading battery level",
                )
                .expect("Error displaying alert");

                thread::sleep(Duration::from_secs(MAX_INTERVAL_IN_SECONDS as u64));
                continue;
            }
        };

        if !is_alert_allowed && (battery_level > 20 || battery_level < 80) {
            is_alert_allowed = true;
        }

        display_alert_if_needed(battery_level, &mut is_alert_allowed);

        let next_execution_in_seconds = get_sleep_seconds(battery_level, MAX_INTERVAL_IN_SECONDS);

        println!(
            "Current battery level: {}%. Checking again in {} seconds.",
            battery_level, next_execution_in_seconds
        );

        thread::sleep(Duration::from_secs(next_execution_in_seconds as u64));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_sleep_seconds_20_percent_battery_is_always_60() {
        let maximum_interval_in_seconds = [0, 20, 50, 100, 200, 300];
        let battery_level = 20;
        let expected_sleep_in_seconds = 60;

        for maximum_interval in maximum_interval_in_seconds {
            println!("Testing maximum interval {}", maximum_interval);
            let sleep_in_seconds = get_sleep_seconds(battery_level, maximum_interval);
            assert_eq!(sleep_in_seconds, expected_sleep_in_seconds);
        }
    }

    #[test]
    fn get_sleep_seconds_80_percent_battery_is_always_60() {
        let maximum_interval_in_seconds = [0, 20, 50, 100, 200, 300];
        let battery_level = 80;
        let expected_sleep_in_seconds = 60;

        for maximum_interval in maximum_interval_in_seconds {
            println!("Testing maximum interval {}", maximum_interval);
            let sleep_in_seconds = get_sleep_seconds(battery_level, maximum_interval);
            assert_eq!(sleep_in_seconds, expected_sleep_in_seconds);
        }
    }

    #[test]
    fn get_sleep_seconds_50_percent_battery_is_always_maximum_interval() {
        let maximum_interval_in_seconds = [0, 20, 50, 100, 200, 300];
        let battery_level = 50;

        for maximum_interval in maximum_interval_in_seconds {
            println!("Testing maximum_interval {}", maximum_interval);
            let sleep_in_seconds = get_sleep_seconds(battery_level, maximum_interval);
            assert_eq!(sleep_in_seconds, maximum_interval);
        }
    }
}
