//ИСХОДНЫЙ КОД ДЛЯ INTEL
use std::fs;
use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent},
    execute,
    style::{self, Color, Colors},
    terminal::{self, ClearType},
};

const BRIGHT_PATH: &str = "/sys/class/backlight/intel_backlight/brightness";
const MAX_BRIGHT_PATH: &str = "/sys/class/backlight/intel_backlight/max_brightness";

struct Config {
    max_bright: u32,
    step_large: u32,
    step_small: u32,
}

fn main() -> io::Result<()> {
    let max_bright = fs::read_to_string(MAX_BRIGHT_PATH)
        .unwrap_or_else(|_| "9600".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(9600);

    let config = Config {
        max_bright,
        step_large: max_bright / 20,    // 5%
        step_small: (max_bright / 100).max(1), // 1%
    };

    let mut current = fs::read_to_string(BRIGHT_PATH)
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
            let _ = fs::write(BRIGHT_PATH, current.to_string());
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
    
    // Текст белый до 50%, на 50% и выше — черный
    let text_color = if val >= cfg.max_bright / 2 { Color::Black } else { Color::White };

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
