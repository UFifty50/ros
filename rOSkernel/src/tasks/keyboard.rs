use crate::kernel::interrupts::CONTROLLER;
use core::pin::Pin;
use core::task::{Context, Poll};
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;
use futures_util::StreamExt;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use pc_keyboard::KeyCode::{CapsLock, NumpadLock, ScrollLock};
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyState, Keyboard, ScancodeSet1};
use ps2::error::ControllerError;
use ps2::flags::{ControllerConfigFlags, KeyboardLedFlags};
use ps2::Controller;
use spin::{Mutex, MutexGuard};

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Uk105Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Uk105Key,
            HandleControl::Ignore
        ));
    static ref KEYBOARD_STATE: Mutex<KeyboardState> = Mutex::new(KeyboardState::new());
}

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::new();
static WAKER: AtomicWaker = AtomicWaker::new();

struct KeyboardState {
    bits: u8,
}

impl KeyboardState {
    pub const fn new() -> KeyboardState {
        KeyboardState { bits: 0b000 }
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .set(ArrayQueue::new(100))
            .expect("Scancode queue already initialized");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.get().expect("Not initialized");
        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

pub(crate) fn addScancode(scancode: u8) {
    if let Some(queue) = SCANCODE_QUEUE.get() {
        if let Err(_) = queue.push(scancode) {
            log::warn!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        log::warn!("WARNING: scancode queue uninitialized");
    }
}

pub async fn printKeypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = KEYBOARD.lock();
    let mut controller = unsafe {
        CONTROLLER.force_unlock();
        CONTROLLER.lock()
    };

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(keyEvent)) = keyboard.add_byte(scancode) {
            log::info!("keyEvent: {:?})", keyEvent);
            if keyEvent.state == KeyState::Down {
                match keyEvent.code {
                    CapsLock => handleLed(&mut controller, KeyboardLedFlags::CAPS_LOCK),
                    NumpadLock => handleLed(&mut controller, KeyboardLedFlags::NUM_LOCK),
                    ScrollLock => handleLed(&mut controller, KeyboardLedFlags::SCROLL_LOCK),
                    _ => {}
                };
            }

            if let Some(key) = keyboard.process_keyevent(keyEvent) {
                match key {
                    DecodedKey::Unicode(character) => log::info!("{}", character),
                    DecodedKey::RawKey(key) => log::info!("{:?}", key),
                }
            }
        }
    }
}

pub fn keyboardInitialize() -> Result<(), ControllerError> {
    let mut controller = CONTROLLER.lock();
    log::info!("controller: {:#?}", controller);
    // Step 3: Disable devices
    controller.disable_keyboard()?;
    log::info!("keyboard disabled");
    controller.disable_mouse()?;

    // Step 4: Flush data buffer
    loop {
        match controller.read_data() {
            Ok(_) => {}
            Err(ControllerError::Timeout) => break,
            Err(e) => return Err(e)
        }
    }

    // Step 5: Set config
    let mut config = controller.read_config()?;
    log::info!("old config: {:#?}", config);
    // Disable interrupts and scancode translation
    config.set(
        ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
            | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT
            | ControllerConfigFlags::ENABLE_TRANSLATE,
        false,
    );
    controller.write_config(config)?;
    log::info!("new config: {:#?}", controller.read_config()?);

    // Step 6: Controller self-test
    controller.test_controller()?;
    log::info!("controller tested");
    // Write config again in case of controller reset
    controller.write_config(config)?;

    // Step 7: Determine if there are 2 devices
    //   let has_mouse = if config.contains(ControllerConfigFlags::DISABLE_MOUSE) {
    //       controller.enable_mouse()?;
    //        config = controller.read_config()?;
    // If mouse is working, this should now be unset
    //      !config.contains(ControllerConfigFlags::DISABLE_MOUSE)
    //   } else {
    //     false
    //   };

    // Step 8: Interface tests
    controller.test_keyboard()?;
    //   let mouse_works = has_mouse && controller.test_mouse().is_ok();

    controller.enable_keyboard()?;
    controller.keyboard().reset_and_self_test().map_err(|_| ControllerError::TestFailed { response: 0 })?;

    let result = controller.keyboard().set_scancode_set(1);
    if let Err(result) = result {
        panic!("Error setting scancode set: {:?}", result);
    }

    controller.keyboard().enable_scanning().map_err(|_| ControllerError::TestFailed { response: 1 })?;

    // Step 9 - 10: Enable and reset devices
    config = controller.read_config()?;
    config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
    //       controller.enable_mouse()?;
    //     config.set(ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT, true);
    //      controller.mouse().enable_data_reporting().unwrap();
    //  }

    // Write last configuration to enable devices and interrupts
    controller.write_config(config)?;
    log::info!("controller config written");

    log::info!("Controller initialized");
    Ok(())
}

fn handleLed(ctrl: &mut MutexGuard<Controller>, key: KeyboardLedFlags) {
    let mut state = KEYBOARD_STATE.lock();

    match key {
        KeyboardLedFlags::CAPS_LOCK => {
            state.bits = state.bits ^ KeyboardLedFlags::CAPS_LOCK.bits();
        }
        KeyboardLedFlags::NUM_LOCK => {
            state.bits = state.bits ^ KeyboardLedFlags::NUM_LOCK.bits();
        }
        KeyboardLedFlags::SCROLL_LOCK => {
            state.bits = state.bits ^ KeyboardLedFlags::SCROLL_LOCK.bits();
        }
        _ => {}
    }

    let b = KeyboardLedFlags::from_bits_truncate(state.bits);
    log::info!("LED: {:?}", b);
    match ctrl.keyboard().set_leds(b) {
        Ok(_) => (),
        Err(e) => log::error!("Error setting led: {:?}", e),
    }
}
