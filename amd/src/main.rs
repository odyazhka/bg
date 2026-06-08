//ИСХОДНЫЙ КОД ДЛЯ AMD
use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use crossterm::{
    cursor,
    event::{self, EnableMouseCapture, DisableMouseCapture, KeyCode, KeyEvent, MouseEventKind},
    execute,
    style::{self, Color, Colors},
    terminal::{self, ClearType},
};

const BRIGHT_PATH_BL0: &str = "/sys/class/backlight/amdgpu_bl0/brightness";
const MAX_BRIGHT_PATH_BL0: &str = "/sys/class/backlight/amdgpu_bl0/max_brightness";
const BRIGHT_PATH_BL1: &str = "/sys/class/backlight/amdgpu_bl1/brightness";
const MAX_BRIGHT_PATH_BL1: &str = "/sys/class/backlight/amdgpu_bl1/max_brightness";

lazy_static! {
    /// Путь к файлу сохранения яркости.
    /// Всегда указывает на ~/.local/bg настоящего пользователя,
    /// независимо от того, запущено ли с sudo.
    static ref SAVE_FILE: PathBuf = {
        let home = if let Ok(sudo_user) = env::var("SUDO_USER") {
            format!("/home/{}", sudo_user)
        } else {
            env::var("HOME").unwrap_or_else(|_| ".".into())
        };

        let mut path = PathBuf::from(home);
        path.push(".local");
        path.push("bg");
        path
    };
}

struct Config {
    max_bright: u32,
    step_large: u32,
    step_small: u32,
}

/// Страж: при любом Drop (в том числе панике или Ctrl-C) сохраняет яркость
struct BrightGuard {
    value: Arc<AtomicU32>,
    bright_path: &'static str,
}

impl Drop for BrightGuard {
    fn drop(&mut self) {
        let v = self.value.load(Ordering::Relaxed);
        save_brightness(v, self.bright_path);
    }
}

fn detect_paths() -> (&'static str, &'static str) {
    if fs::metadata(BRIGHT_PATH_BL0).is_ok() {
        (BRIGHT_PATH_BL0, MAX_BRIGHT_PATH_BL0)
    } else {
        (BRIGHT_PATH_BL1, MAX_BRIGHT_PATH_BL1)
    }
}

fn save_brightness(val: u32, bright_path: &str) {
    if let Some(parent) = SAVE_FILE.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&*SAVE_FILE, val.to_string());
    let _ = fs::write(bright_path, val.to_string());
}

fn main() -> io::Result<()> {
    let (bright_path, max_bright_path) = detect_paths();

    let max_bright = fs::read_to_string(max_bright_path)
        .unwrap_or_else(|_| "9600".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(9600);

    let config = Config {
        max_bright,
        step_large: max_bright / 20,
        step_small: (max_bright / 100).max(1),
    };

    let init = fs::read_to_string(bright_path)
        .unwrap_or_else(|_| "500".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(500);

    let shared = Arc::new(AtomicU32::new(init));

    let _guard = BrightGuard { value: Arc::clone(&shared), bright_path };

    {
        let shared_ctrlc = Arc::clone(&shared);
        ctrlc::set_handler(move || {
            let v = shared_ctrlc.load(Ordering::Relaxed);
            save_brightness(v, bright_path);
            let _ = execute!(
                io::stdout(),
                DisableMouseCapture,
                cursor::Show,
                terminal::Clear(ClearType::All)
            );
            let _ = terminal::disable_raw_mode();
            std::process::exit(0);
        })
        .expect("Не удалось установить обработчик Ctrl-C");
    }

    let mut current = init;
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide, EnableMouseCapture, terminal::Clear(ClearType::All))?;

    loop {
        draw_ui(&mut stdout, current, &config)?;

        match event::read()? {
            // --- Клавиатура ---
            event::Event::Key(KeyEvent { code, .. }) => {
                match code {
                    KeyCode::Up => {
                        let next = if current == 1 { config.step_large } else { current + config.step_large };
                        current = (next / config.step_large * config.step_large).min(config.max_bright);
                    }
                    KeyCode::Down => {
                        current = if current > config.step_large { current - config.step_large } else { 1 };
                    }
                    KeyCode::Right => {
                        let next = if current == 1 { config.step_small } else { current + config.step_small };
                        current = (next / config.step_small * config.step_small).min(config.max_bright);
                    }
                    KeyCode::Left => {
                        current = if current > config.step_small { current - config.step_small } else { 1 };
                    }
                    // Q / q / Й / й / Esc — выход
                    KeyCode::Char('q') | KeyCode::Char('Q') |
                    KeyCode::Char('й') | KeyCode::Char('Й') |
                    KeyCode::Esc => break,
                    _ => {}
                }

                save_brightness(current, bright_path);
                shared.store(current, Ordering::Relaxed);
            }

            // --- Колёсико мыши ---
            event::Event::Mouse(mouse_event) => {
                match mouse_event.kind {
                    MouseEventKind::ScrollUp => {
                        let next = current + config.step_large;
                        current = next.min(config.max_bright);
                    }
                    MouseEventKind::ScrollDown => {
                        current = if current > config.step_large { current - config.step_large } else { 1 };
                    }
                    _ => {}
                }

                save_brightness(current, bright_path);
                shared.store(current, Ordering::Relaxed);
            }

            _ => {}
        }
    }

    execute!(stdout, DisableMouseCapture, cursor::Show, terminal::Clear(ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn draw_ui(stdout: &mut io::Stdout, val: u32, cfg: &Config) -> io::Result<()> {
    let percent = val * 100 / cfg.max_bright;

    let gray_index = (val * 23 / cfg.max_bright) as u8;
    let color_code = 232 + gray_index;

    let text_color = if val <= cfg.max_bright / 2 { Color::White } else { Color::Black };

    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        style::SetColors(Colors::new(text_color, Color::AnsiValue(color_code))),
        terminal::Clear(ClearType::All)
    )?;

    writeln!(stdout, "\r\n УПРАВЛЕНИЕ ЯРКОСТЬЮ (↑/↓/←/→/колёсико/Q)\r")?;
    writeln!(stdout, "\r\n Текущее значение: {} ({}%)\r", val, percent)?;
    writeln!(stdout, "\r\n Сохранение: {}\r", SAVE_FILE.display())?;
    stdout.flush()
}
