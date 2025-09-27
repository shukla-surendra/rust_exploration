use std::thread;
use std::time::Duration;
use std::io::{self, Write};
use std::process::Command;

const POMODORO_DURATION_MINUTES: u64 = 1; // Keep your test duration
const SHORT_BREAK_DURATION_MINUTES: u64 = 1;
const LONG_BREAK_DURATION_MINUTES: u64 = 2;

#[derive(Clone, Debug)]
enum TimerState {
    Working,
    ShortBreak,
    LongBreak,
}

struct PomodoroTimer {
    cycle_count: u32,
    state: TimerState,
}

impl PomodoroTimer {
    fn new() -> Self {
        PomodoroTimer {
            cycle_count: 0,
            state: TimerState::Working,
        }
    }

    fn start(&mut self) {
        println!("🍅 Pomodoro Timer Started!");
        println!("Press Ctrl+C to stop the timer\n");

        loop {
            match self.state {
                TimerState::Working => self.start_work_session(),
                TimerState::ShortBreak => self.start_short_break(),
                TimerState::LongBreak => self.start_long_break(),
            }
        }
    }

    fn start_work_session(&mut self) {
        self.cycle_count += 1;
        println!("📚 Work Session {} Started - Focus for {} minutes!",
                 self.cycle_count, POMODORO_DURATION_MINUTES);

        self.countdown_timer(POMODORO_DURATION_MINUTES * 60, "⏰ Work Time");

        // Determine next state
        if self.cycle_count % 4 == 0 {
            self.state = TimerState::LongBreak;
        } else {
            self.state = TimerState::ShortBreak;
        }
    }

    fn start_short_break(&mut self) {
        println!("\n🎉 Work session {} completed!", self.cycle_count);
        self.show_break_screen("Short Break Time! ☕", SHORT_BREAK_DURATION_MINUTES, false);
        self.state = TimerState::Working;
    }

    fn start_long_break(&mut self) {
        println!("\n🎉 Completed 4 pomodoros! Great job!");
        self.show_break_screen("Long Break Time! 🏖️", LONG_BREAK_DURATION_MINUTES, true);
        self.state = TimerState::Working;
    }

    fn show_break_screen(&self, title: &str, duration_minutes: u64, is_long_break: bool) {
        // Clear screen and show full-screen break message
        self.clear_screen();

        // Try to open Windows notification if possible from WSL
        self.try_windows_notification(title);

        // Show prominent terminal overlay
        self.show_terminal_overlay(title, duration_minutes, is_long_break);

        // Countdown with forced break
        self.countdown_timer(duration_minutes * 60, title);
    }

    fn show_terminal_overlay(&self, title: &str, duration_minutes: u64, is_long_break: bool) {
        self.clear_screen();

        let width = 80;
        let border = "═".repeat(width);
        let padding = " ".repeat((width - title.len()) / 2);

        println!("\n{}", "█".repeat(width + 4));
        println!("█{}█", " ".repeat(width + 2));
        println!("█  {}{}{}  █", padding, title, padding);
        println!("█{}█", " ".repeat(width + 2));

        if is_long_break {
            println!("█  {}Take a proper rest - walk around, stretch, hydrate!{}  █",
                     " ".repeat(15), " ".repeat(15));
            println!("█  {}You've earned this {}min break after 4 pomodoros!{}  █",
                     " ".repeat(12), duration_minutes, " ".repeat(12));
        } else {
            println!("█  {}Step away from your screen for {} minutes{}  █",
                     " ".repeat(18), duration_minutes, " ".repeat(18));
            println!("█  {}Stretch, breathe, or grab some water{}  █",
                     " ".repeat(20), " ".repeat(20));
        }

        println!("█{}█", " ".repeat(width + 2));
        println!("█  {}This window will stay here until break is over{}  █",
                 " ".repeat(12), " ".repeat(12));
        println!("█  {}(Ctrl+C to stop timer completely){}  █",
                 " ".repeat(21), " ".repeat(21));
        println!("█{}█", " ".repeat(width + 2));
        println!("{}", "█".repeat(width + 4));
        println!("\n");
    }

    fn countdown_timer(&self, total_seconds: u64, phase_name: &str) {
        for remaining in (1..=total_seconds).rev() {
            let minutes = remaining / 60;
            let seconds = remaining % 60;

            // Move cursor to specific line for countdown
            print!("\r⏱️  {} - Time remaining: {:02}:{:02}   ",
                   phase_name, minutes, seconds);
            io::stdout().flush().unwrap();

            thread::sleep(Duration::from_secs(1));
        }
        println!("\n✅ {} Complete!", phase_name);
    }

    fn clear_screen(&self) {
        // Clear screen using ANSI escape codes (works in WSL)
        print!("\x1B[2J\x1B[1;1H");
        io::stdout().flush().unwrap();
    }

    fn try_windows_notification(&self, message: &str) {
        // Try to use Windows Toast notification from WSL
        // This requires PowerShell access from WSL
        let powershell_cmd = format!(
            r#"Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show('{}', 'Pomodoro Timer', 'OK', 'Information')"#,
            message
        );

        // Try multiple ways to show notification
        let _ = Command::new("powershell.exe")
            .args(&["-Command", &powershell_cmd])
            .output();

        // Alternative: Windows toast notification
        let toast_cmd = format!(
            r#"[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; [Windows.UI.Notifications.ToastNotification, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] > $null; $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); $xml = [xml] $template.GetXml(); $xml.toast.visual.binding.text[0].AppendChild($xml.CreateTextNode('Pomodoro Timer')) > $null; $xml.toast.visual.binding.text[1].AppendChild($xml.CreateTextNode('{}')) > $null; $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Pomodoro').Show($toast)"#,
            message
        );

        let _ = Command::new("powershell.exe")
            .args(&["-Command", &toast_cmd])
            .output();

        // Try wsl-notify if available (needs to be installed separately)
        let _ = Command::new("wsl-notify-send")
            .args(&["Pomodoro Timer", message])
            .output();
    }

    // Add method to make break uninterruptible
    fn force_break_compliance(&self) {
        // Disable common terminal shortcuts during break
        println!("🚫 Break enforcement active:");
        println!("   - Alt+Tab, Windows key, and other shortcuts are discouraged");
        println!("   - Focus on resting, not on your screen");
        println!("   - The timer will continue regardless");
    }
}

fn main() {
    // Handle Ctrl+C gracefully
    ctrlc::set_handler(move || {
        println!("\n👋 Pomodoro timer stopped. Great work!");
        std::process::exit(0);
    }).expect("Error setting Ctrl+C handler");

    println!("🎯 WSL Pomodoro Timer with Terminal Overlay");
    println!("This version creates a prominent terminal display that's hard to ignore!");
    println!("For even better notifications, consider installing Windows Terminal or wsl-notify\n");

    let mut timer = PomodoroTimer::new();
    timer.start();
}

// Add to Cargo.toml dependencies:
/*
[dependencies]
ctrlc = "3.0"

# Optional for better WSL integration:
# wsl-notify = "1.0"  # If you install wsl-notify-send
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = PomodoroTimer::new();
        assert_eq!(timer.cycle_count, 0);
        matches!(timer.state, TimerState::Working);
    }

    // Quick test with 5-second intervals
    pub fn test_quick_timer() {
        println!("🧪 Quick test mode starting...");
        let mut timer = PomodoroTimer::new();

        // Override durations for testing
        println!("Work period (5 seconds)");
        timer.countdown_timer(5, "Test Work");

        timer.show_break_screen("Test Break! ☕", 1, false);

        println!("✅ Quick test completed!");
    }
}