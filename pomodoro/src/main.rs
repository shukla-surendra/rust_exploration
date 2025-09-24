use std::thread;
use std::time::Duration;

#[cfg(target_os = "windows")]
use winapi::um::winuser::{MessageBoxW, MB_OK, MB_ICONINFORMATION};
#[cfg(target_os = "windows")]
use winapi::um::winnt::LPCWSTR;
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

#[cfg(target_os = "linux")]
use std::process::Command;

#[cfg(target_os = "macos")]
use std::process::Command;

const POMODORO_DURATION_MINUTES: u64 = 1;
const BREAK_DURATION_MINUTES: u64 = 1;

struct PomodoroTimer {
    cycle_count: u32,
    is_break_time: bool,
}

impl PomodoroTimer {
    fn new() -> Self {
        PomodoroTimer {
            cycle_count: 0,
            is_break_time: false,
        }
    }

    fn start(&mut self) {
        println!("🍅 Pomodoro Timer Started!");
        println!("Press Ctrl+C to stop the timer\n");

        loop {
            if self.is_break_time {
                self.start_break();
            } else {
                self.start_work_session();
            }
        }
    }

    fn start_work_session(&mut self) {
        self.cycle_count += 1;
        println!("📚 Work Session {} Started - Focus for {} minutes!", 
                 self.cycle_count, POMODORO_DURATION_MINUTES);
        
        // Wait for 25 minutes
        thread::sleep(Duration::from_secs(POMODORO_DURATION_MINUTES * 60));
        
        // Show popup notification
        self.show_popup("Pomodoro Complete! 🍅", 
                       &format!("Work session {} completed! Time for a break.", self.cycle_count));
        
        self.is_break_time = true;
    }

    fn start_break(&mut self) {
        let break_duration = if self.cycle_count % 4 == 0 {
            println!("☕ Long Break Time! Take 15-30 minutes to recharge.");
            15 // Long break after every 4 pomodoros
        } else {
            println!("☕ Short Break Time! Take {} minutes to rest.", BREAK_DURATION_MINUTES);
            BREAK_DURATION_MINUTES
        };

        thread::sleep(Duration::from_secs(break_duration * 60));
        
        self.show_popup("Break Over! 🚀", "Ready to start the next work session?");
        self.is_break_time = false;
    }

    #[cfg(target_os = "windows")]
    fn show_popup(&self, title: &str, message: &str) {
        unsafe {
            let title_wide: Vec<u16> = OsStr::new(title)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let message_wide: Vec<u16> = OsStr::new(message)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            MessageBoxW(
                std::ptr::null_mut(),
                message_wide.as_ptr() as LPCWSTR,
                title_wide.as_ptr() as LPCWSTR,
                MB_OK | MB_ICONINFORMATION,
            );
        }
    }

    #[cfg(target_os = "linux")]
    fn show_popup(&self, title: &str, message: &str) {
        // Try zenity first (most common)
        let result = Command::new("zenity")
            .args(&["--info", "--title", title, "--text", message])
            .output();

        if result.is_err() {
            // Fallback to notify-send
            let _ = Command::new("notify-send")
                .args(&[title, message])
                .output();
        }
    }

    #[cfg(target_os = "macos")]
    fn show_popup(&self, title: &str, message: &str) {
        let script = format!(
            r#"display notification "{}" with title "{}""#,
            message, title
        );
        
        let _ = Command::new("osascript")
            .args(&["-e", &script])
            .output();
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    fn show_popup(&self, title: &str, message: &str) {
        println!("🔔 {}: {}", title, message);
        println!("(Native popups not supported on this platform)");
    }
}

fn main() {
    // Handle Ctrl+C gracefully
    ctrlc::set_handler(move || {
        println!("\n👋 Pomodoro timer stopped. Great work!");
        std::process::exit(0);
    }).expect("Error setting Ctrl+C handler");

    let mut timer = PomodoroTimer::new();
    timer.start();
}

// For testing - shorter durations
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = PomodoroTimer::new();
        assert_eq!(timer.cycle_count, 0);
        assert_eq!(timer.is_break_time, false);
    }

    // Quick test function with 10-second intervals
    pub fn test_quick_timer() {
        let mut timer = PomodoroTimer::new();
        println!("🧪 Test mode: 10-second intervals");
        
        for i in 1..=3 {
            println!("Test cycle {}: Working...", i);
            thread::sleep(Duration::from_secs(10));
            timer.show_popup("Test Complete", &format!("Test cycle {} done!", i));
            
            println!("Test cycle {}: Break...", i);
            thread::sleep(Duration::from_secs(5));
            timer.show_popup("Break Over", "Back to work!");
        }
    }
}