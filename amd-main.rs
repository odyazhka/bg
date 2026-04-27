// ИСХОДНЫЙ КОД ДЛЯ AMD
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use crossterm::{
    cursor,
    event::{self, KeyCode, KeyEvent},
    execute,
    style::{self, Color, Colors},
    terminal::{self, ClearType},
};

struct Config {
    max_bright: u32,
    step_large: u32,
    step_small: u32,
    bright_path: PathBuf,
}

// Функция для динамического поиска интерфейса управления яркостью AMD
fn find_amd_backlight_dir() -> Option<PathBuf> {
    let entries = fs::read_dir("/sys/class/backlight/").ok()?;
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        // Ищем папку, которая начинается с amdgpu_bl
        if name.starts_with("amdgpu_bl") {
            return Some(entry.path());
        }
    }
    None
}

fn main() -> io::Result<()> {
    // 1. Ищем путь динамически
    let base_path = match find_amd_backlight_dir() {
        Some(path) => path,
        None => {
            println!("Ошибка: Устройство управления подсветкой AMD (amdgpu_bl*) не найдено.");
            return Ok(());
        }
    };

    let max_bright_path = base_path.join("max_brightness");
    let bright_path = base_path.join("brightness");

    // 2. Читаем максимальную яркость
    let max_bright = fs::read_to_string(&max_bright_path)
        .unwrap_or_else(|_| "255".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(255);

    // 3. Настраиваем шаги с защитой от нуля (.max(1))
    let config = Config {
        max_bright,
        step_large: (max_bright / 20).max(1),    // 5%
        step_small: (max_bright / 100).max(1),   // 1%
        bright_path: bright_path.clone(),
    };

    // 4. Читаем текущую яркость
    let mut current = fs::read_to_string(&bright_path)
        .unwrap_or_else(|_| "128".to_string())
        .trim()
        .parse::<u32>()
        .unwrap_or(128);

    // 5. Запуск интерфейса
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide, terminal::Clear(ClearType::All))?;

    loop {
        draw_ui(&mut stdout, current, &config)?;

        if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
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
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => break,
                _ => {}
            }

            // Записываем в систему (ошибки игнорируются, если нет прав)
            let _ = fs::write(&config.bright_path, current.to_string());
        }
    }

    // 6. Выход
    execute!(stdout, cursor::Show, terminal::Clear(ClearType::All))?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn draw_ui(stdout: &mut io::Stdout, val: u32, cfg: &Config) -> io::Result<()> {
    let percent = val * 100 / cfg.max_bright;
    
    let gray_index = (val * 23 / cfg.max_bright) as u8;
    let color_code = 232 + gray_index;
    
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
