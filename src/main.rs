mod discovery;
mod metrics;
mod types;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::env;
use std::io;
use std::time::{Duration, Instant};

const UI_REFRESH_INTERVAL_MS: u64 = 33;
const METRICS_UPDATE_INTERVAL_MS: u64 = 250;

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let json_mode = args.contains(&String::from("--json"));

    if json_mode {
        run_json_mode()
    } else {
        run_interactive_mode()
    }
}

fn run_json_mode() -> Result<(), io::Error> {
    let use_fake_data = std::env::var("IBTOP_FAKE_DATA").is_ok();

    let adapters = if use_fake_data {
        discovery::fake::generate_fake_adapters()
    } else {
        let real_adapters = discovery::discover_adapters();
        if real_adapters.is_empty() && std::env::var("IBTOP_DEMO").is_ok() {
            discovery::fake::generate_fake_adapters()
        } else {
            real_adapters
        }
    };

    let json_output = serde_json::to_string_pretty(&adapters)?;
    println!("{json_output}");

    Ok(())
}

fn run_interactive_mode() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let use_fake_data = std::env::var("IBTOP_FAKE_DATA").is_ok();
    let mut metrics = metrics::MetricsCollector::new();

    let ui_refresh_duration = Duration::from_millis(UI_REFRESH_INTERVAL_MS);
    let metrics_update_interval = Duration::from_millis(METRICS_UPDATE_INTERVAL_MS);

    let mut last_metrics_update = Instant::now();
    let mut adapters = Vec::new();

    loop {
        let now = Instant::now();

        if now.duration_since(last_metrics_update) >= metrics_update_interval {
            adapters = if use_fake_data {
                discovery::fake::generate_fake_adapters()
            } else {
                let real_adapters = discovery::discover_adapters();
                if real_adapters.is_empty() && std::env::var("IBTOP_DEMO").is_ok() {
                    discovery::fake::generate_fake_adapters()
                } else {
                    real_adapters
                }
            };

            metrics.update(&adapters);
            last_metrics_update = now;
        }

        terminal.draw(|f| ui::draw(f, &adapters, &metrics))?;

        let timeout = ui_refresh_duration.saturating_sub(now.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(())
                    }
                    KeyCode::Char('r') => {
                        last_metrics_update = Instant::now()
                            .checked_sub(metrics_update_interval)
                            .unwrap_or_else(Instant::now);
                    }
                    _ => {}
                }
            }
        }
    }
}
