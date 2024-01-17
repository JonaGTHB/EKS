use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use inquire::CustomType;
use rand::{thread_rng, Rng};
use rfd::FileDialog;
use spinners::{Spinner, Spinners};
use std::io::{self, BufRead, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::{fs::File, io::stdout};
use textplots::{Chart, Plot, Shape};

const MIN_DELAY_MS: u64 = 50;
const MAX_DELAY_MS: u64 = 400;

const EXTRA_TYPING_PROBABILITY: f64 = 0.05;
const EXTRA_TYPING_CHARS_MIN: u64 = 1;
const EXTRA_TYPING_CHARS_MAX: u64 = 3;

const SLEEPY_PROBABILITY: f64 = 0.05;
const SLEEPY_MINUTES_MIN: u64 = 2;
const SLEEPY_MINUTES_MAX: u64 = 10;

enum Key {
    Return,
    Backspace,
}

impl Key {
    fn to_str(&self) -> &'static str {
        match self {
            Key::Return => "return",
            Key::Backspace => "backspace",
        }
    }
}

fn main() -> io::Result<()> {
    let file = FileDialog::new().pick_file();

    let selected_file_path: Option<PathBuf> = match file {
        Some(file_handle) => Some(file_handle.to_path_buf()),
        None => {
            println!("No file was selected");
            return Ok(());
        }
    };

    let path = selected_file_path.unwrap();

    let _min_delay_ms: u64 = CustomType::new("Enter the minimum delay (in ms)")
        .with_error_message("Please enter a valid number")
        .prompt()
        .unwrap();

    let _max_delay_ms: u64 = CustomType::new("Enter the maximum delay (in ms)")
        .with_error_message("Please enter a valid number")
        .prompt()
        .unwrap();

    let _extra_typing_probability: f64 = CustomType::new("Enter the extra typing probability")
        .with_error_message("Please enter a valid floating point number")
        .prompt()
        .unwrap();

    let _extra_typing_chars_min: u64 = CustomType::new("Enter the minimum extra typed characters")
        .with_error_message("Please enter a valid number")
        .prompt()
        .unwrap();

    let _extra_typing_chars_max: u64 = CustomType::new("Enter the maximum extra typed characters")
        .with_error_message("Please enter a valid number")
        .prompt()
        .unwrap();

    let mut file = File::open(path)?;
    let reader = io::BufReader::new(&file);

    let total_chars = reader
        .lines()
        .map(|line| line.map(|l| l.chars().count() + 1).unwrap_or(0))
        .sum::<usize>();

    println!("Total chars: {}", total_chars);
    println!("Waiting 5 seconds, press Ctrl+C to cancel");

    let (lowest_runtime, average_runtime, worst_case_runtime) = calculate_runtimes(total_chars);

    plot_runtimes(lowest_runtime, average_runtime, worst_case_runtime);

    print_runtimes(lowest_runtime, average_runtime, worst_case_runtime);

    println!("");
    println!("");

    thread::sleep(Duration::from_secs(5));

    file.seek(SeekFrom::Start(0))?;

    let reader = io::BufReader::new(file);

    write(reader)?;

    Ok(())
}

fn print_runtimes(lowest_runtime: f64, average_runtime: f64, worst_case_runtime: f64) {
    println!("Estimated lowest runtime: {} seconds", lowest_runtime);
    println!("Estimated average runtime: {} seconds", average_runtime);
    println!(
        "Estimated worst-case runtime: {} seconds",
        worst_case_runtime
    );
}

fn write(reader: io::BufReader<File>) -> io::Result<()> {
    for line in reader.lines() {
        // _clear_terminal()?;
        let line = line?;

        if line.trim().is_empty() {
            flip_do_break();
        } else {
            println!("Printing line: {}", line);
        }

        for ch in line.chars() {
            flip_do_typo()?;

            let delay = random_delay();
            thread::sleep(delay);
            type_letter(&ch.to_string())?;
        }

        thread::sleep(random_delay() * 2);
        let _ = type_key(Key::Return);
    }
    Ok(())
}

fn flip_do_typo() -> Result<(), io::Error> {
    Ok(if thread_rng().gen_bool(EXTRA_TYPING_PROBABILITY) {
        let extra_chars = thread_rng().gen_range(EXTRA_TYPING_CHARS_MIN..=EXTRA_TYPING_CHARS_MAX);
        let spinner_message = format!("Faking a typo by typing {} extra chars", extra_chars);
        let mut spinner = Spinner::new(Spinners::Dots9, spinner_message);
        for _ in 0..extra_chars {
            let extra_char = thread_rng().gen_range(b'a'..=b'z') as char;
            type_letter(&extra_char.to_string())?;
            thread::sleep(random_delay());
        }
        for _ in 0..extra_chars {
            type_key(Key::Backspace)?;
            thread::sleep(random_delay());
        }
        spinner.stop_with_symbol("✓");
    })
}

fn flip_do_break() {
    if thread_rng().gen_bool(SLEEPY_PROBABILITY) {
        let wait_minutes = thread_rng().gen_range(SLEEPY_MINUTES_MIN..=SLEEPY_MINUTES_MAX);
        let spinner_message = format!("Line is empty, sleeping for {} minutes...", wait_minutes);

        let _spinner = Spinner::new(Spinners::Dots9, spinner_message);
        thread::sleep(Duration::from_secs(wait_minutes * 60));
    }
}

fn type_key(key: Key) -> io::Result<()> {
    Command::new("wtype").arg("-k").arg(key.to_str()).status()?;
    Ok(())
}

fn type_letter(key: &str) -> io::Result<()> {
    Command::new("wtype").arg(key).status()?;
    Ok(())
}

fn random_delay() -> Duration {
    let mut rng = thread_rng();
    let delay = rng.gen_range(MIN_DELAY_MS..MAX_DELAY_MS);
    Duration::from_millis(delay)
}

fn calculate_runtimes(total_chars: usize) -> (f64, f64, f64) {
    let lowest_runtime_seconds = total_chars as f64 * MIN_DELAY_MS as f64 / 1000.0;

    let average_delay_ms = (MIN_DELAY_MS + MAX_DELAY_MS) / 2;
    let average_extra_chars = (EXTRA_TYPING_CHARS_MIN + EXTRA_TYPING_CHARS_MAX) / 2;
    let typo_adjustment =
        EXTRA_TYPING_PROBABILITY * average_extra_chars as f64 * 2.0 * average_delay_ms as f64;
    let average_runtime_seconds =
        total_chars as f64 * (average_delay_ms as f64 + typo_adjustment) / 1000.0;

    let max_typo_adjustment =
        EXTRA_TYPING_PROBABILITY * EXTRA_TYPING_CHARS_MAX as f64 * 2.0 * MAX_DELAY_MS as f64;
    let worst_case_runtime_seconds =
        total_chars as f64 * (MAX_DELAY_MS as f64 + max_typo_adjustment) / 1000.0;

    (
        lowest_runtime_seconds,
        average_runtime_seconds,
        worst_case_runtime_seconds,
    )
}

fn plot_runtimes(lowest_runtime: f64, average_runtime: f64, worst_case_runtime: f64) {
    let data = [
        (0.0f32, 0.0f32),
        (1.0f32, lowest_runtime as f32),
        (2.0f32, average_runtime as f32),
        (3.0f32, worst_case_runtime as f32),
    ];

    println!("");
    println!("Runtime Estimates:");
    println!("");
    Chart::new(180, 60, 0.0, 4.0)
        .lineplot(&Shape::Bars(&data))
        .nice();
}

fn _clear_terminal() -> io::Result<()> {
    execute!(stdout(), Clear(ClearType::All))?;
    Ok(())
}
