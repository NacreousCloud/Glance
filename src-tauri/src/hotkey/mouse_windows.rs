#![cfg(target_os = "windows")]

//! Windows mouse hotkey listener via `SetWindowsHookExW(WH_MOUSE_LL)`.
//!
//! A low-level mouse hook procedure runs in the context of the thread
//! that installed it (and only if that thread runs a message pump), so
//! we spawn a dedicated thread, install the hook there, and drain
//! messages via `GetMessageW`. The hook callback is a `extern "system"`
//! function and cannot capture environment, so the sender + bindings
//! are stashed in process-static `OnceCell`s the callback reads.
//!
//! Scope mirrors the macOS path: only buttons 3+ (middle / back /
//! forward) can fire a binding so the global hotkey cannot collide with
//! ordinary left/right clicks. Modifier state is sampled with
//! `GetAsyncKeyState`.

use crate::hotkey::{SharedBindings, TriggerEvent};
use crate::settings::HotkeyTrigger;
use std::sync::OnceLock;
use parking_lot::Mutex;
use std::thread;
use tokio::sync::mpsc::UnboundedSender;
use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    HHOOK, MSG, MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_MBUTTONDOWN, WM_XBUTTONDOWN, XBUTTON1, XBUTTON2,
};

const MOD_CMD: u8 = 1;
const MOD_CTRL: u8 = 2;
const MOD_SHIFT: u8 = 4;
const MOD_ALT: u8 = 8;

static TX: OnceLock<Mutex<UnboundedSender<TriggerEvent>>> = OnceLock::new();
static BINDINGS: OnceLock<SharedBindings> = OnceLock::new();

fn read_modifiers() -> u8 {
    let mut out = 0u8;
    unsafe {
        // GetAsyncKeyState's high-order bit (0x8000) is set when the key is
        // currently pressed.
        if (GetAsyncKeyState(VK_LWIN.0 as i32) as u16) & 0x8000 != 0
            || (GetAsyncKeyState(VK_RWIN.0 as i32) as u16) & 0x8000 != 0
        {
            out |= MOD_CMD;
        }
        if (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16) & 0x8000 != 0 {
            out |= MOD_CTRL;
        }
        if (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16) & 0x8000 != 0 {
            out |= MOD_SHIFT;
        }
        if (GetAsyncKeyState(VK_MENU.0 as i32) as u16) & 0x8000 != 0 {
            out |= MOD_ALT;
        }
    }
    out
}

fn dispatch(button: u8) {
    let modifiers = read_modifiers();
    let Some(bindings) = BINDINGS.get() else { return };
    let Some(tx) = TX.get() else { return };
    let snap = bindings.lock();
    for b in snap.iter() {
        if let HotkeyTrigger::Mouse {
            button: bb,
            modifiers: mm,
        } = &b.trigger
        {
            if *bb == button && *mm == modifiers {
                let _ = tx.lock().send(TriggerEvent {
                    binding_id: b.id.clone(),
                    menu_mode: b.menu_mode.clone(),
                });
                break;
            }
        }
    }
}

unsafe extern "system" fn mouse_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code < 0 {
        return CallNextHookEx(HHOOK(std::ptr::null_mut()), n_code, w_param, l_param);
    }
    let msg = w_param.0 as u32;
    match msg {
        WM_MBUTTONDOWN => dispatch(3),
        WM_XBUTTONDOWN => {
            // mouseData high-order word is XBUTTON1 (back, 4) or XBUTTON2 (forward, 5).
            let info = &*(l_param.0 as *const MSLLHOOKSTRUCT);
            let xbtn = (info.mouseData >> 16) as u16;
            if xbtn == XBUTTON1.0 {
                dispatch(4);
            } else if xbtn == XBUTTON2.0 {
                dispatch(5);
            }
        }
        _ => {}
    }
    CallNextHookEx(HHOOK(std::ptr::null_mut()), n_code, w_param, l_param)
}

/// Same external shape as the macOS counterpart so lib.rs can call
/// `MouseMonitor::start` on both platforms.
pub struct MouseMonitor;

impl MouseMonitor {
    pub fn start(bindings: SharedBindings, trigger_tx: UnboundedSender<TriggerEvent>) -> Self {
        if BINDINGS.set(bindings).is_err() {
            tracing::warn!("MouseMonitor::start called twice; ignoring second install");
            return Self;
        }
        let _ = TX.set(Mutex::new(trigger_tx));

        thread::Builder::new()
            .name("glance-mouse-hook".into())
            .spawn(|| unsafe {
                let module = match GetModuleHandleW(None) {
                    Ok(h) => h,
                    Err(e) => {
                        tracing::error!(error = ?e, "GetModuleHandleW failed");
                        return;
                    }
                };
                let hook = match SetWindowsHookExW(
                    WH_MOUSE_LL,
                    Some(mouse_proc),
                    HINSTANCE(module.0),
                    0,
                ) {
                    Ok(h) => h,
                    Err(e) => {
                        tracing::error!(error = ?e, "SetWindowsHookExW(WH_MOUSE_LL) failed");
                        return;
                    }
                };
                tracing::info!("Windows low-level mouse hook installed");

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                // Process-lifetime: in practice the loop only exits at
                // process shutdown. The hook is also cleaned up by the
                // OS when the owning process dies.
                drop(hook);
            })
            .expect("spawn mouse hook thread");
        Self
    }
}
