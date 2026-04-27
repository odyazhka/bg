//ИСХОДНЫЙ КОД ДЛЯ AMD
use lazy_static::lazy_static;
use std::env;
use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent},
    execute,
    style::{self, Color, Colors},
    terminal::{self, ClearType},
};

const BRIGHT_PATH_BL0: &str = "/sys/class/backlight/amdgpu_bl0/brightness";
const MAX_BRIGHT_PATH_BL0: &str = "/sys/class/backlight/amdgpu_bl0/max_brightness";
const BRIGHT_PATH_BL1: &str = "/sys/class/backlight/amdgpu_bl1/brightness";
const MAX_BRIGHT_PATH_BL1: &str = "/sys/class/backlight/amdgpu_bl1/max_brightness";

lazy_static! {
    // Эта переменная вычислится один раз при первом вызове
    static ref SAVE_FILE: PathBuf = {
        let home = env::var("HOME").unwrap_or_else(|_| ".".into());
        let mut path = PathBuf::from(home);
        path.push(".bg");
        path
    };
}

struct Config {
    max_bright: u32,
    step_large: u32,
    step_small: u32,
}

fn detect_paths() -> (&'static str, &'static str) {
    if fs::metadata(BRIGHT_PATH_BL0).is_ok() {
        (BRIGHT_PATH_BL0, MAX_BRIGHT_PATH_BL0)
    } else {
        (BRIGHT_PATH_BL1, MAX_BRIGHT_PATH_BL1)
    }
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
        step_large: max_bright / 20,    // 5%
        step_small: (max_bright / 100).max(1), // 1%
    };

    let mut current = fs::read_to_string(bright_path)
        .unwrap_or_else(|_| "500".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(500);

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    loop {
        draw_ui(&mut stdout, current, &config)?;

        if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Up => {
                    // Если на 1, прыгаем ровно на 480. Если выше — выравниваем по сетке 480.
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
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                _ => {}
            }

            // Записываем в систему
            let _ = fs::write(bright_path, current.to_string());
            let _ = fs::write(&*SAVE_FILE, current.to_string());
        }
    }

    execute!(stdout, cursor::Show, terminal::Clear(ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn draw_ui(stdout: &mut io::Stdout, val: u32, cfg: &Config) -> io::Result<()> {
    // В Rust деление u32 на u32 автоматически дает целое число
    let percent = val * 100 / cfg.max_bright;
    
    let gray_index = (val * 23 / cfg.max_bright) as u8;
    let color_code = 232 + gray_index;
    
    // Текст белый до 50%, выше — черный
    let text_color = if val <= cfg.max_bright / 2 { Color::White } else { Color::Black };

    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        style::SetColors(Colors::new(text_color, Color::AnsiValue(color_code))),
        terminal::Clear(ClearType::All)
    )?;

    writeln!(stdout, "\r\n УПРАВЛЕНИЕ ЯРКОСТЬЮ (↑/↓/←/→/Q)\r")?;
    writeln!(stdout, "\r\n Текущее значение: {} ({}%)\r", val, percent)?;
    stdout.flush()
}
