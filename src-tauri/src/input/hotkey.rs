use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_graphics::event::{
    CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CallbackResult,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub type HotkeyCallback = Box<dyn Fn(HotkeyEvent) + Send + 'static>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HotkeyEvent {
    RecordStart,
    RecordStop,
}

pub fn start_fn_key_monitor(callback: HotkeyCallback) -> FnKeyMonitor {
    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    let handle = std::thread::spawn(move || {
        let fn_down = Arc::new(AtomicBool::new(false));
        let fn_down_clone = fn_down.clone();

        let tap = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![CGEventType::FlagsChanged],
            move |_proxy, _event_type, event| {
                let flags = event.get_flags();
                let fn_pressed = flags.contains(CGEventFlags::CGEventFlagSecondaryFn);
                let was_down = fn_down_clone.load(Ordering::SeqCst);

                if fn_pressed && !was_down {
                    fn_down_clone.store(true, Ordering::SeqCst);
                    callback(HotkeyEvent::RecordStart);
                } else if !fn_pressed && was_down {
                    fn_down_clone.store(false, Ordering::SeqCst);
                    callback(HotkeyEvent::RecordStop);
                }

                CallbackResult::Keep
            },
        );

        match tap {
            Ok(tap) => {
                let loop_source = tap
                    .mach_port()
                    .create_runloop_source(0)
                    .expect("Failed to create run loop source");
                let run_loop = CFRunLoop::get_current();
                run_loop.add_source(&loop_source, unsafe { kCFRunLoopCommonModes });
                tap.enable();

                while running_clone.load(Ordering::SeqCst) {
                    CFRunLoop::run_in_mode(
                        unsafe { kCFRunLoopCommonModes },
                        std::time::Duration::from_millis(100),
                        false,
                    );
                }
            }
            Err(_) => {
                eprintln!(
                    "Failed to create event tap. \
                     Ensure Wipr has Input Monitoring permission."
                );
            }
        }
    });

    FnKeyMonitor {
        running,
        _handle: handle,
    }
}

pub struct FnKeyMonitor {
    running: Arc<AtomicBool>,
    _handle: std::thread::JoinHandle<()>,
}

impl Drop for FnKeyMonitor {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
