use AppConfig;

use web_sys::Event;
use web_sys::{
    MouseEvent,
    KeyboardEvent,
    HtmlCanvasElement,
    HtmlBodyElement,
    Window,
    EventTarget,
    HtmlEvent,
    Document
};

use std::cell::RefCell;
use std::rc::Rc;

use AppEvent;

pub struct App {
    window: HtmlCanvasElement,
    window_o:Window,
    pub events: Rc<RefCell<Vec<AppEvent>>>,
    device_pixel_ratio: f32,
}

use super::events;

macro_rules! map_event {
    ($events:expr, $x:ident, $y:ident, $ee:ident, $e:expr, $prevent:expr) => {{
        let events = $events.clone();
        move |$ee: $x| {
            if $prevent {
                $ee.prevent_default();
            }
            events.borrow_mut().push(AppEvent::$y($e));
        }
    }};

    ($events:expr, $x:ident, $y:ident, $e:expr) => {{
        let events = $events.clone();
        move |_: $x| {
            events.borrow_mut().push(AppEvent::$y($e));
        }
    }};
}

// In browser request full screen can only called under event handler.
// So basically this function is useless at this moment.
#[allow(dead_code)]
fn request_full_screen(canvas: &HtmlCanvasElement) {
    canvas.request_fullscreen().unwrap();
}

impl App {
    pub fn new(config: AppConfig) -> App {
        
        if config.headless {
            // Right now we did not support headless in web.
            unimplemented!();
        }
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("document should have a body");
        let canvas: HtmlCanvasElement = document.create_element("canvas")
            .unwrap();
        let real_to_css_pixels = Window::device_pixel_ratio() as u32;
        canvas.set_width(config.size.0 * real_to_css_pixels);
        canvas.set_height(config.size.1 * real_to_css_pixels);
        canvas.set_tab_index(1);

        if !config.show_cursor {
            /*    
            js! {
                @{&canvas}.style.cursor="none";
            };
            */
        }

        let device_pixel_ratio: f64 = Window::device_pixel_ratio();
        body.append_child(&canvas);
        canvas.focus().unwrap();

        if config.fullscreen {
            println!("Webgl do not support with_screen.");
        }

        let mut app = App {
            window: canvas,
            events: Rc::new(RefCell::new(vec![])),
            device_pixel_ratio: device_pixel_ratio as f32,
        };
        app.setup_listener();

        app
    }

    fn setup_listener(&mut self) {
        let canvas: &HtmlCanvasElement = self.canvas();

        canvas.add_event_listener_with_callback(map_event!{
            self.events,
            MouseDownEvent,
            MouseDown,
            e,
            MouseEvent::button(),
            false
        });
        canvas.add_event_listener_with_callback(map_event!{
            self.events,
            MouseUpEvent,
            MouseUp,
            e,
            MouseEvent::button(),
            true
        });

        canvas.add_event_listener_with_callback({
            let canvas = canvas.clone();
            let canvas_x: f64 = canvas.get_bounding_client_rect().left();
            let canvas_y: f64 = canvas.get_bounding_client_rect().top();
            map_event!{
                self.events,
                MouseMoveEvent,
                MousePos,
                e,
                (e.client_x() as f64 - canvas_x,e.client_y() as f64 - canvas_y),
                true
            }
        });

        canvas.add_event_listener(map_event!{
            self.events,
            KeyDownEvent,
            KeyDown,
            e,
            KeyboardEvent::key_code(),
            true
        });

        // canvas.add_event_listener(map_event!{
        //     self.events,
        //     KeypressEvent,
        //     KeyPress,
        //     e,
        //     events::KeyPressEvent {
        //         code: e.code()
        //     }
        // });

        canvas.add_event_listener(map_event!{
            self.events,
            KeyUpEvent,
            KeyUp,
            e,
            KeyboardEvent::key_code(),
            true
        });

        canvas.add_event_listener({
            let canvas = canvas.clone();

            map_event!{
                self.events,
                ResizeEvent,
                Resized,
                (canvas.offset_width() as u32, canvas.offset_height() as u32)
            }
        });
    }

    pub fn print<T: Into<JsValue>>(msg: T) {
        web_sys::console::log_1(&msg.into());
    }

    pub fn exit() {}

    pub fn get_params() -> Vec<String> {
        let params = js!{ return window.location.search.substring(1).split("&"); };
        params.try_into().unwrap()
    }

    pub fn hidpi_factor(&self) -> f32 {
        return self.device_pixel_ratio;
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.window
    }

    pub fn run_loop<F>(mut self, mut callback: F)
    where
        F: 'static + FnMut(&mut Self) -> (),
    {
        window().request_animation_frame(move |_t: f64| {
            callback(&mut self);
            self.events.borrow_mut().clear();
            self.run_loop(callback);
        });
    }

    pub fn poll_events<F>(&mut self, callback: F) -> bool
    where
        F: FnOnce(&mut Self) -> (),
    {
        callback(self);
        self.events.borrow_mut().clear();

        true
    }

    pub fn run<F>(self, callback: F)
    where
        F: 'static + FnMut(&mut Self) -> (),
    {
        self.run_loop(callback);

        //stdweb::event_loop();
    }

    pub fn set_fullscreen(&mut self, _b: bool) {
        // unimplemented!();
    }
}

pub fn now(&self) -> f64 {
    // perforamce now is in ms
    // https://developer.mozilla.org/en-US/docs/Web/API/Performance/now
    self.window_o.performance().now()/1000.0
}
