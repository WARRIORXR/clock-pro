//! Clock Pro — Rust terminal port of the single-file web clock
//! (`index.html` in the parent folder): digital clock, stopwatch,
//! countdown timer and world clock.
//!
//! Run with `cargo run`. Type commands as shown; Ctrl+C always works.

use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Local, Timelike, Utc};
use chrono_tz::Tz;

const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const DIM: &str = "\x1b[2m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";
const CLEAR: &str = "\x1b[2J\x1b[H";

/// Background thread feeding every stdin line into a channel.
fn spawn_reader() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
                let _ = tx.send("q".to_string()); // EOF behaves like quit
                break;
            }
            if tx.send(line.trim().to_string()).is_err() {
                break;
            }
        }
    });
    rx
}

fn drain(rx: &Receiver<String>) -> Option<String> {
    let mut latest = None;
    loop {
        match rx.try_recv() {
            Ok(msg) => latest = Some(msg),
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => break,
        }
    }
    latest
}

// ── Modes ───────────────────────────────────────────────────

fn digital_clock(rx: &Receiver<String>) {
    loop {
        let now = Local::now();
        print!(
            "{CLEAR}{BOLD}   {:02}:{:02}:{:02}{RESET}\n\n   {} \n\n {DIM}type q + Enter to go back{RESET}",
            now.hour(),
            now.minute(),
            now.second(),
            now.format("%A, %d %B %Y"),
        );
        let _ = io::stdout().flush();
        if matches!(drain(rx).as_deref(), Some("q") | Some("quit") | Some("exit")) {
            return;
        }
        thread::sleep(Duration::from_millis(250));
    }
}

fn stopwatch(rx: &Receiver<String>) {
    let mut accumulated = Duration::ZERO;
    let mut started: Option<Instant> = None;
    loop {
        let elapsed = accumulated + started.map(|s| s.elapsed()).unwrap_or_default();
        let state = if started.is_some() { "running" } else { "paused" };
        print!(
            "\r {BOLD}{:>02}:{:>02}.{:02}{RESET} [{state}] {DIM}s=start/stop r=reset q=done{RESET}   ",
            elapsed.as_secs() / 60,
            elapsed.as_secs() % 60,
            elapsed.subsec_millis() / 10,
        );
        let _ = io::stdout().flush();
        match drain(rx).as_deref() {
            Some("s") => match started.take() {
                Some(s) => accumulated += s.elapsed(),
                None => started = Some(Instant::now()),
            },
            Some("r") => {
                accumulated = Duration::ZERO;
                started = None;
            }
            Some("q") | Some("quit") | Some("exit") => {
                let total = accumulated + started.map(|s| s.elapsed()).unwrap_or_default();
                println!(
                    "\n {GREEN}Final: {:02}:{:02}.{:02}{RESET}",
                    total.as_secs() / 60,
                    total.as_secs() % 60,
                    total.subsec_millis() / 10
                );
                return;
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn timer(rx: &Receiver<String>) {
    println!(" {DIM}Countdown as MM:SS (e.g. 05:00):{RESET}");
    let input = match rx.recv() {
        Ok(line) => line,
        Err(_) => return,
    };
    let (mm, ss) = match input.split_once(':') {
        Some((m, s)) => (
            m.trim().parse::<u64>().unwrap_or(0),
            s.trim().parse::<u64>().unwrap_or(0),
        ),
        None => (input.trim().parse::<u64>().unwrap_or(0), 0),
    };
    let total = Duration::from_secs(mm * 60 + ss);
    if total.is_zero() {
        println!(" {RED}Nothing to count down.{RESET}");
        return;
    }
    let deadline = Instant::now() + total;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        print!(
            "\r ⏳ {BOLD}{:02}:{:02}{RESET} remaining {DIM}(q = cancel){RESET}   ",
            remaining.as_secs() / 60,
            remaining.as_secs() % 60,
        );
        let _ = io::stdout().flush();
        if remaining.is_zero() {
            println!("\r {YELLOW}{BOLD}⏰ TIME'S UP!{RESET}\x07                    ");
            return;
        }
        if matches!(drain(rx).as_deref(), Some("q") | Some("quit") | Some("exit")) {
            println!("\r {DIM}Timer cancelled.{RESET}           ");
            return;
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn world_clock() {
    let zones: [(&str, &str); 7] = [
        ("Delhi", "Asia/Kolkata"),
        ("London", "Europe/London"),
        ("New York", "America/New_York"),
        ("Tokyo", "Asia/Tokyo"),
        ("Dubai", "Asia/Dubai"),
        ("Sydney", "Australia/Sydney"),
        ("UTC", "UTC"),
    ];
    println!();
    let now_utc = Utc::now();
    for (city, tz_name) in zones {
        let tz: Tz = tz_name.parse().expect("valid tz");
        let t = now_utc.with_timezone(&tz);
        println!(
            " {BOLD}{:<10}{RESET} {}  {DIM}{}{RESET}",
            city,
            t.format("%H:%M:%S"),
            t.format("%a %d %b")
        );
    }
    println!();
}

fn main() {
    println!("{BOLD}⏰ Clock Pro — Rust Edition{RESET}");
    let rx = spawn_reader();
    loop {
        println!("  {CYAN}1{RESET} digital clock   {CYAN}2{RESET} stopwatch   {CYAN}3{RESET} timer   {CYAN}4{RESET} world clock   {CYAN}q{RESET} quit");
        print!("> ");
        let _ = io::stdout().flush();
        let choice = match rx.recv() {
            Ok(line) => line,
            Err(_) => break,
        };
        match choice.as_str() {
            "1" => digital_clock(&rx),
            "2" => stopwatch(&rx),
            "3" => timer(&rx),
            "4" => world_clock(),
            "q" | "quit" | "exit" => break,
            _ => println!(" {DIM}choose 1-4 or q{RESET}"),
        }
    }
    println!("Bye! 👋");
}
