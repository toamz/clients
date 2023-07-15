use anyhow::{Ok, Result};
use widestring::U16CStr;
use std::{mem::size_of};
use windows::Win32::{UI::{
    Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_TYPE, KEYBDINPUT, self, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY, VkKeyScanExA, GetKeyboardLayout},
    WindowsAndMessaging::{GetForegroundWindow, GetMessageExtraInfo, GetWindowTextW, GetWindow, GW_HWNDNEXT, IsWindowVisible, GetWindowThreadProcessId}, TextServices,
}, Foundation::HWND, System::{ProcessStatus::GetProcessImageFileNameW, Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION}}};


const MAX_WINDOW_TITLE_LENGTH: usize = 1024;
const APP_PREFIX: &str = "windowsapp://";

unsafe fn set_virtual_key(key: VIRTUAL_KEY, pressed: bool) {
    let extra_info = GetMessageExtraInfo();
    let extra_info = extra_info.0.unsigned_abs();
    let a = vec![ INPUT {
        r#type: INPUT_TYPE { 0: 1 },
        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: if pressed {KeyboardAndMouse::KEYBD_EVENT_FLAGS(0)} else {KEYEVENTF_KEYUP},
                time: 0,
                dwExtraInfo: extra_info,
            },
        },
    }];
    SendInput(&a, size_of::<INPUT>() as i32);
}

pub fn release_modifiers() -> Result<()> {
    let keys = vec![
        KeyboardAndMouse::VK_SHIFT,
        KeyboardAndMouse::VK_MENU,
        KeyboardAndMouse::VK_CONTROL,
        KeyboardAndMouse::VK_CAPITAL,
    ];

    for vk in keys {
        unsafe {
            set_virtual_key(vk, false);
        }
    }
    Ok(())
}

unsafe fn send_char(c: char) {
    let extra_info = GetMessageExtraInfo();
    let extra_info = extra_info.0.unsigned_abs();
    let a = vec![ INPUT {
        r#type: INPUT_TYPE { 0: 1 },
        Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
            ki: KEYBDINPUT {
                wVk: KeyboardAndMouse::VIRTUAL_KEY(0), // virtual-key code - zero for KEYEVENTF_UNICODE
                wScan: c as u16,               // Unicode character
                dwFlags: KEYEVENTF_UNICODE,
                time: 0,
                dwExtraInfo: extra_info,
            },
        },
    }];
    SendInput(&a, size_of::<INPUT>() as i32);
}

unsafe fn send_char_as_virtual(c: char, keyboard: TextServices::HKL) {
    let vk_with_modifiers = VkKeyScanExA(c as u8, keyboard);
    let vk = VIRTUAL_KEY((vk_with_modifiers & 0xFF) as u16);

    if vk_with_modifiers == -1 {
        return send_char(c);
    }

    let modifiers = vk_with_modifiers >> 8;
    let shift = (modifiers & 1) != 0;
    let ctrl = (modifiers & 2) != 0;
    let alt = (modifiers & 4) != 0;

    if shift {set_virtual_key(KeyboardAndMouse::VK_SHIFT, true);}
    if ctrl {set_virtual_key(KeyboardAndMouse::VK_CONTROL, true);}
    if alt {set_virtual_key(KeyboardAndMouse::VK_MENU, true);}

    set_virtual_key(vk, true);
    set_virtual_key(vk, false);

    if shift {set_virtual_key(KeyboardAndMouse::VK_SHIFT, false);}
    if ctrl {set_virtual_key(KeyboardAndMouse::VK_CONTROL, false);}
    if alt {set_virtual_key(KeyboardAndMouse::VK_MENU, false);}
}

fn window_title(hwnd: HWND) -> String {
    let mut buf = [0; MAX_WINDOW_TITLE_LENGTH];
    unsafe {
        GetWindowTextW(hwnd, &mut buf);
    }
    
    let str = U16CStr::from_slice_truncate(&buf).expect("Expected window to have name").to_string_lossy();
    str
}

pub fn send_text(data: &str) -> Result<()> {
    unsafe {
        let typing_window = GetForegroundWindow();
        release_modifiers().ok();
        let mut pid = 0;
        let thread_id = GetWindowThreadProcessId(typing_window, Some(&mut pid));
        let keyboard = GetKeyboardLayout(thread_id);
        for c in data.chars() {
            if GetForegroundWindow() != typing_window {
                return Ok(());
            }
            send_char_as_virtual(c, keyboard);
        }
    }
    Ok(())
}

pub fn send_login(username: &str, pass: &str) -> Result<()> {
    send_text(username).ok();
    unsafe {
        set_virtual_key(KeyboardAndMouse::VK_TAB,true);
        set_virtual_key(KeyboardAndMouse::VK_TAB,false);
    }
    send_text(pass).ok();
    Ok(())
}

unsafe fn window_executable(hwnd: HWND) -> Option<String> {
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));

    if pid == 0 {
        return None;
    }

    let mut buf = [0; MAX_WINDOW_TITLE_LENGTH];

    let h_process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid);
    if h_process.is_err() {
        return None;
    }
    
    GetProcessImageFileNameW(h_process.unwrap(), &mut buf);

    let str = U16CStr::from_slice_truncate(&buf).ok();
    if str.is_none() {
        return None;
    }

    let str_decoded = str.unwrap().to_string_lossy();
    let (_, name) = str_decoded.rsplit_once("\\").unwrap_or(("", str_decoded.as_str()));

    Some(name.to_owned())
}

pub fn active_window_url() -> Result<String> {
    unsafe {
        let fg_window = GetForegroundWindow();
        let title = window_title(fg_window);
        let executable = window_executable(fg_window);
        Ok(format!("{}{}/{}", APP_PREFIX, executable.unwrap_or("unknown".to_owned()), title))
    }
}

pub fn next_window_url() -> Result<String> {
    unsafe {
        let mut cur_hwnd = GetForegroundWindow();
        loop {
            cur_hwnd = GetWindow(cur_hwnd, GW_HWNDNEXT);
            if IsWindowVisible(cur_hwnd).as_bool() {
                break;
            }
        }
        let title = window_title(cur_hwnd);
        let executable = window_executable(cur_hwnd);
        Ok(format!("{}{}/{}", APP_PREFIX, executable.unwrap_or("unknown".to_owned()), title))
    }
}
