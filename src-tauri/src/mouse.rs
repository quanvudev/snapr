use crate::Action;
use std::fmt;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use winit::event_loop::EventLoopProxy;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_CONTROL, VK_MENU, VK_SHIFT};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, MSG, MSLLHOOKSTRUCT, SetWindowsHookExW, WH_MOUSE_LL, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1, XBUTTON2,
};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};

const UNBOUND: u32 = 0;
const BUTTON_BITS: u32 = 8;
const BUTTON_MASK: u32 = (1 << BUTTON_BITS) - 1;
const HIGH_WORD_SHIFT: u32 = 16;
const KEY_DOWN_BIT: i16 = i16::MIN;
const SWALLOW: LRESULT = 1;
const HC_ACTION: i32 = 0;

const CTRL: u8 = 1;
const ALT: u8 = 2;
const SHIFT: u8 = 4;
const MODIFIERS: [(u8, &str, i32); 3] = [
    (CTRL, "Ctrl", VK_CONTROL as i32),
    (ALT, "Alt", VK_MENU as i32),
    (SHIFT, "Shift", VK_SHIFT as i32),
];

const BUTTONS: [(u8, &str); 3] = [(1, "Mouse3"), (2, "Mouse4"), (3, "Mouse5")];
const MIDDLE: u8 = 1;
const BACK: u8 = 2;
const FORWARD: u8 = 3;

static BOUND: AtomicU32 = AtomicU32::new(UNBOUND);
static PENDING_UP: AtomicU32 = AtomicU32::new(UNBOUND);
static PROXY: OnceLock<Mutex<EventLoopProxy<Action>>> = OnceLock::new();

#[derive(Clone, Copy, PartialEq)]
pub struct Binding {
    button: u8,
    mods: u8,
}

impl Binding {
    pub fn parse(text: &str) -> Option<Self> {
        let mut button = None;
        let mut mods = 0;
        for token in text.split('+') {
            if let Some(&(bit, _, _)) = MODIFIERS.iter().find(|(_, name, _)| name.eq_ignore_ascii_case(token)) {
                mods |= bit;
            } else {
                let &(id, _) = BUTTONS.iter().find(|(_, name)| name.eq_ignore_ascii_case(token))?;
                if button.replace(id).is_some() {
                    return None;
                }
            }
        }
        button.map(|button| Self { button, mods })
    }

    fn pack(self) -> u32 {
        self.button as u32 | (self.mods as u32) << BUTTON_BITS
    }
}

impl fmt::Display for Binding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (bit, name, _) in MODIFIERS {
            if self.mods & bit != 0 {
                write!(f, "{name}+")?;
            }
        }
        let name = BUTTONS.iter().find(|(id, _)| *id == self.button).map_or("", |(_, name)| name);
        f.write_str(name)
    }
}

pub fn bind(binding: Option<Binding>) {
    BOUND.store(binding.map_or(UNBOUND, Binding::pack), Ordering::Relaxed);
}

fn held_modifiers() -> u8 {
    MODIFIERS
        .iter()
        .filter(|(_, _, vk)| unsafe { GetAsyncKeyState(*vk) } & KEY_DOWN_BIT != 0)
        .fold(0, |mods, (bit, _, _)| mods | bit)
}

fn classify(msg: u32, data: u32) -> Option<(u8, bool)> {
    let x_button = || match (data >> HIGH_WORD_SHIFT) as u16 {
        x if x == XBUTTON1 => Some(BACK),
        x if x == XBUTTON2 => Some(FORWARD),
        _ => None,
    };
    match msg {
        WM_MBUTTONDOWN => Some((MIDDLE, true)),
        WM_MBUTTONUP => Some((MIDDLE, false)),
        WM_XBUTTONDOWN => x_button().map(|b| (b, true)),
        WM_XBUTTONUP => x_button().map(|b| (b, false)),
        _ => None,
    }
}

unsafe extern "system" fn hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION {
        let data = unsafe { (*(lparam as *const MSLLHOOKSTRUCT)).mouseData };
        if let Some((button, down)) = classify(wparam as u32, data) {
            if down {
                let bound = BOUND.load(Ordering::Relaxed);
                if bound != UNBOUND
                    && (bound & BUTTON_MASK) as u8 == button
                    && (bound >> BUTTON_BITS) as u8 == held_modifiers()
                {
                    PENDING_UP.store(button as u32, Ordering::Relaxed);
                    if let Some(proxy) = PROXY.get() {
                        let _ = proxy.lock().unwrap().send_event(Action::Ring);
                    }
                    return SWALLOW;
                }
            } else if PENDING_UP.compare_exchange(button as u32, UNBOUND, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
                return SWALLOW;
            }
        }
    }
    unsafe { CallNextHookEx(null_mut(), code, wparam, lparam) }
}

pub fn start(proxy: EventLoopProxy<Action>) {
    if PROXY.set(Mutex::new(proxy)).is_err() {
        return;
    }
    std::thread::spawn(|| unsafe {
        if SetWindowsHookExW(WH_MOUSE_LL, Some(hook), null_mut(), 0).is_null() {
            return;
        }
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {}
    });
}
