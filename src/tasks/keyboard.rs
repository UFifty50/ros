use futures_util::StreamExt;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, KeyState};
use pc_keyboard::KeyCode::{NumpadLock, CapsLock, ScrollLock};
use ps2::Controller;
use ps2::error::ControllerError;
use ps2::flags::{ControllerConfigFlags, KeyboardLedFlags};
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use spin::{MutexGuard, Mutex};
use core::pin::Pin;
use core::task::{Context, Poll};
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;
use crate::kernel::interrupts::CONTROLLER;
use crate::{print, println};


lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Uk105Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(ScancodeSet1::new(), layouts::Uk105Key, HandleControl::Ignore)
    );

    static ref KEYBOARD_STATE: Mutex<KeyboardState> = Mutex::new(
            KeyboardState::new()
        );
}

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

struct KeyboardState {
    bits: u8
}

impl KeyboardState {
    pub const fn new() -> KeyboardState {
        KeyboardState {
            bits: 0b000
        }
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("Scancode queue already initialized");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("Not initialized");
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
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
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
            println!("keyEvent: {:?})", keyEvent);
            if keyEvent.state == KeyState::Down {
                match keyEvent.code {
                    CapsLock => handleLed(&mut controller, KeyboardLedFlags::CAPS_LOCK),
                    NumpadLock => handleLed(&mut controller, KeyboardLedFlags::NUM_LOCK),
                    ScrollLock => handleLed(&mut controller, KeyboardLedFlags::SCROLL_LOCK),
                    _ => {},
                };
            }

            println!("code: {:?}", keyEvent.code);

            if let Some(key) = keyboard.process_keyevent(keyEvent) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}

pub fn keyboardInitialize() -> Result<(), ControllerError> {
    let mut controller = CONTROLLER.lock();

    // Step 3: Disable devices
    controller.disable_keyboard()?;
  //  controller.disable_mouse()?;

    // Step 4: Flush data buffer
    let _ = controller.read_data();

    // Step 5: Set config
    let mut config = controller.read_config()?;
    // Disable interrupts and scancode translation
    config.set(
        ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT
            | ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT
            | ControllerConfigFlags::ENABLE_TRANSLATE,
        false,
    );
    controller.write_config(config)?;

    // Step 6: Controller self-test
    controller.test_controller()?;
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
    // Disable mouse. If there's no mouse, this is ignored
    controller.disable_mouse()?;

    // Step 8: Interface tests
    let keyboard_works = controller.test_keyboard().is_ok();
 //   let mouse_works = has_mouse && controller.test_mouse().is_ok();

    // Step 9 - 10: Enable and reset devices
    config = controller.read_config()?;
    if keyboard_works {
        controller.enable_keyboard()?;
        config.set(ControllerConfigFlags::DISABLE_KEYBOARD, false);
        config.set(ControllerConfigFlags::ENABLE_KEYBOARD_INTERRUPT, true);
     //   let x = controller.keyboard().reset_and_self_test();
     //   print!("\nwe here {:#?}", x);
    }
 //   if mouse_works {
 //       controller.enable_mouse()?;
  //      config.set(ControllerConfigFlags::DISABLE_MOUSE, false);
   //     config.set(ControllerConfigFlags::ENABLE_MOUSE_INTERRUPT, true);
   //     controller.mouse().reset_and_self_test().unwrap();
        // This will start streaming events from the mouse
  //      controller.mouse().enable_data_reporting().unwrap();
  //  }

    // Write last configuration to enable devices and interrupts
    controller.write_config(config)?;

    let result = controller.keyboard().set_scancode_set(1);
    if let Err(result) = result {
        panic!("Error setting scancode set: {:?}", result);
    }

    println!("Controller initialized");
    Ok(())
}

fn handleLed(ctrl: &mut MutexGuard<Controller>, key: KeyboardLedFlags) {
    let mut state = KEYBOARD_STATE.lock();

    match key {
        KeyboardLedFlags::CAPS_LOCK => {
            state.bits = state.bits ^ KeyboardLedFlags::CAPS_LOCK.bits();
        },
        KeyboardLedFlags::NUM_LOCK => {
            state.bits = state.bits ^ KeyboardLedFlags::NUM_LOCK.bits();
        },
        KeyboardLedFlags::SCROLL_LOCK => {
            state.bits = state.bits ^ KeyboardLedFlags::SCROLL_LOCK.bits();
        },
        _ => {}
    }

    let b = KeyboardLedFlags::from_bits_truncate(state.bits);
    println!("LED: {:?}", b);
    match ctrl.keyboard().set_leds(b) {
        Ok(_) => (),
        Err(e) => println!("Error setting led: {:?}", e),
    }
}

