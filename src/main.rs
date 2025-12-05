use windows::{
    core::Result,
    Win32::{
        Foundation::{LPARAM, LRESULT, WPARAM},
        UI::{
            Input::KeyboardAndMouse::{
                INPUT, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_SHIFT, VK_MENU, GetKeyState, SendInput,
            },
            WindowsAndMessaging::{
                HHOOK, KBDLLHOOKSTRUCT, MSG,
                HC_ACTION, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
                CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx
            },
        },
    },
};

static mut KEYBOARD_HOOK: HHOOK = HHOOK(std::ptr::null_mut());

// 用于识别 SendInput 发送的事件，避免递归
const MAGIC_EXTRA_INFO: usize = 0x1234_5678_8765_4321;

// 键位常量
const VK_OEM_2: VIRTUAL_KEY = VIRTUAL_KEY(191); // '/'
const VK_DIVIDE: VIRTUAL_KEY = VIRTUAL_KEY(111); // 小键盘 '/'

fn main() -> Result<()> {
    unsafe {
        // 设置键盘钩子
        let result = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(low_level_keyboard_proc),
            None,
            0
        );

        match result {
            Ok(hook) => KEYBOARD_HOOK = hook,
            Err(e) => return Err(e),
        }

        // 消息循环
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // 清理钩子
        if !KEYBOARD_HOOK.0.is_null() {
            let _ = UnhookWindowsHookEx(KEYBOARD_HOOK);
        }
    }

    Ok(())
}

unsafe extern "system" fn low_level_keyboard_proc(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode == HC_ACTION as i32 {
        // 检查指针有效性
        if lparam.0 == 0 {
            return CallNextHookEx(Some(KEYBOARD_HOOK), ncode, wparam, lparam);
        }

        let hook = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

        // 如果是自己用 SendInput 发送的事件，则跳过，避免递归
        if hook.dwExtraInfo == MAGIC_EXTRA_INFO {
            return CallNextHookEx(Some(KEYBOARD_HOOK), ncode, wparam, lparam);
        }

        // 检查修饰键（Ctrl / Shift / Alt）状态
        if GetKeyState(VK_CONTROL.0 as i32) >= 0 && GetKeyState(VK_SHIFT.0 as i32) >= 0 && GetKeyState(VK_MENU.0 as i32) >= 0 {
            // 检查是否斜杠键
            if hook.vkCode == VK_OEM_2.0 as u32 {
                // 处理 keydown / keyup / syskeydown / syskeyup
                let is_keydown = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
                let is_keyup = wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize;

                let mut inputs: [INPUT; 1] = std::mem::zeroed();

                if is_keydown {
                    inputs[0].r#type = INPUT_KEYBOARD;
                    inputs[0].Anonymous.ki.wVk = VK_DIVIDE;
                    inputs[0].Anonymous.ki.dwFlags = KEYBD_EVENT_FLAGS(0);
                    inputs[0].Anonymous.ki.dwExtraInfo = MAGIC_EXTRA_INFO;
                } else if is_keyup {
                    inputs[0].r#type = INPUT_KEYBOARD;
                    inputs[0].Anonymous.ki.wVk = VK_DIVIDE;
                    inputs[0].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
                    inputs[0].Anonymous.ki.dwExtraInfo = MAGIC_EXTRA_INFO;
                }

                // 发送输入，并检查是否成功
                if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == 0 {
                    // 失败时不拦截，避免输入被“吞掉”
                    return CallNextHookEx(Some(KEYBOARD_HOOK), ncode, wparam, lparam);
                }

                // 成功发送后拦截原始斜杠键
                return LRESULT(1);
            }
        }
    }

    CallNextHookEx(Some(KEYBOARD_HOOK), ncode, wparam, lparam)
}
