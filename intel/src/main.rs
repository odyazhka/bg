//ИСХОДНЫЙ КОД ДЛЯ INTEL
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

const BRIGHT_PATH: &str = "/sys/class/backlight/intel_backlight/brightness";
const MAX_BRIGHT_PATH: &str = "/sys/class/backlight/intel_backlight/max_brightness";

lazy_static! {
    /// Путь к файлу сохранения яркости.
    /// Всегда указывает на ~/.local/bg настоящего пользователя,
    /// независимо от того, запущено ли с sudo.
    /// При запуске через sudo SUDO_USER содержит имя реального пользователя.
    static ref SAVE_FILE: PathBuf = {
        // Если запущено через sudo — берём домашнюю директорию реального пользователя
        let home = if let Ok(sudo_user) = env::var("SUDO_USER") {
            // /home/<user> — стандарт для Linux
            // Более надёжно: читаем из /etc/passwd через getpwnam, но для простоты используем /home/
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
}

impl Drop for BrightGuard {
    fn drop(&mut self) {
        let v = self.value.load(Ordering::Relaxed);
        save_brightness(v);
    }
}

fn save_brightness(val: u32) {
    // Создаём директорию ~/.local/ если её нет
    if let Some(parent) = SAVE_FILE.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&*SAVE_FILE, val.to_string());
    let _ = fs::write(BRIGHT_PATH, val.to_string());
}

fn main() -> io::Result<()> {
    let max_bright = fs::read_to_string(MAX_BRIGHT_PATH)
        .unwrap_or_else(|_| "9600".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(9600);

    let config = Config {
        max_bright,
        step_large: max_bright / 20,
        step_small: (max_bright / 100).max(1),
    };

    let init = fs::read_to_string(BRIGHT_PATH)
        .unwrap_or_else(|_| "500".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(500);

    // Атомик — shared между основным потоком и потоком Ctrl-C
    let shared = Arc::new(AtomicU32::new(init));

    // Страж живёт до конца main, Drop вызовется в любом случае
    let _guard = BrightGuard { value: Arc::clone(&shared) };

    // Ctrl-C: восстанавливаем терминал и выходим — Drop стража сохранит файл
    {
        let shared_ctrlc = Arc::clone(&shared);
        ctrlc::set_handler(move || {
            let v = shared_ctrlc.load(Ordering::Relaxed);
            save_brightness(v);
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

                save_brightness(current);
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

                save_brightness(current);
                shared.store(current, Ordering::Relaxed);
            }

            _ => {}
        }
    }

    execute!(stdout, DisableMouseCapture, cursor::Show, terminal::Clear(ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
    // Здесь _guard выходит из области видимости → Drop → финальное сохранение
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
