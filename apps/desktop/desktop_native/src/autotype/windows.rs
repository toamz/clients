use anyhow::{Ok, Result};
use widestring::U16CStr;
use std::{mem::size_of};
use windows::Win32::UI::{
    Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_TYPE, KEYBDINPUT, self, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VkKeyScanA, VIRTUAL_KEY},
    WindowsAndMessaging::{GetForegroundWindow, GetMessageExtraInfo, GetWindowTextW},
};

const MAX_WINDOW_TITLE_LENGTH: usize = 1024;

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

unsafe fn send_char_as_virtual(c: char) {
    let vk_with_modifiers = VkKeyScanA(c as u8);
    let vk = VIRTUAL_KEY((vk_with_modifiers & 0xFF) as u16);
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

pub fn send_text(data: &str) -> Result<()> {
    unsafe {
        let typing_window = GetForegroundWindow();
        release_modifiers().ok();
        for c in data.chars() {
            if GetForegroundWindow() != typing_window {
                return Ok(());
            }
            send_char_as_virtual(c);
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

pub fn active_window_title() -> Result<String> {
    let mut buf = [0; MAX_WINDOW_TITLE_LENGTH];
    unsafe {
        let fg_window = GetForegroundWindow();
        GetWindowTextW(fg_window, &mut buf);
    }
    
    let str = U16CStr::from_slice_truncate(&buf).expect("Expected window to have name").to_string_lossy();
    Ok(str)
}
