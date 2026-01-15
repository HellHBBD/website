use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::window;

#[derive(Clone)]
pub struct GestureHandlers {
    pub on_touch_start: EventHandler<Event<TouchData>>,
    pub on_touch_move: EventHandler<Event<TouchData>>,
    pub on_touch_end: EventHandler<Event<TouchData>>,
    pub on_mouse_down: EventHandler<Event<MouseData>>,
    pub on_mouse_move: EventHandler<Event<MouseData>>,
    pub on_mouse_up: EventHandler<Event<MouseData>>,
    pub on_mouse_leave: EventHandler<Event<MouseData>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GestureState {
    pub start_x: f64,
    pub start_y: f64,
    pub last_x: f64,
    pub last_y: f64,
    pub drag_x: f64,
    pub is_swiping: bool,
    pub is_vertical_scroll_locked: bool,
    last_click_time: f64,
    last_touch_end_time: f64, // To prevent emulated mouse events
    min_swipe_distance: f64,
    double_click_interval: f64,
}

impl Default for GestureState {
    fn default() -> Self {
        Self {
            start_x: 0.0,
            start_y: 0.0,
            last_x: 0.0,
            last_y: 0.0,
            drag_x: 0.0,
            is_swiping: false,
            is_vertical_scroll_locked: false,
            last_click_time: 0.0,
            last_touch_end_time: 0.0,
            min_swipe_distance: 80.0,
            double_click_interval: 300.0, // 300ms
        }
    }
}

fn handle_gesture_end<F1, F2, F3>(
    state: &mut GestureState,
    performance: &Option<web_sys::Performance>,
    on_swipe_left: &Rc<RefCell<F1>>,
    on_swipe_right: &Rc<RefCell<F2>>,
    on_double_click: &Rc<RefCell<F3>>,
)
where
    F1: FnMut(),
    F2: FnMut(),
    F3: FnMut(),
{
    if !state.is_swiping || state.is_vertical_scroll_locked {
        state.is_swiping = false;
        state.drag_x = 0.0;
        return;
    }

    let distance_x = state.last_x - state.start_x;
    let distance_y = state.last_y - state.start_y;

    let (swiped, is_swipe_right) = if distance_x.abs() > state.min_swipe_distance
        && distance_x.abs() > distance_y.abs() * 3.0
    {
        if distance_x > 0.0 {
            on_swipe_right.borrow_mut()();
            (true, true)
        } else {
            on_swipe_left.borrow_mut()();
            (true, false)
        }
    } else {
        if let Some(p) = &performance {
            let now = p.now();
            let movement = distance_x.hypot(distance_y);
            if now - state.last_click_time < state.double_click_interval && movement < 10.0 {
                on_double_click.borrow_mut()();
                state.last_click_time = 0.0; // Reset after double click
            } else {
                state.last_click_time = now;
            }
        }
        (false, false)
    };

    state.is_swiping = false;
    if !swiped || is_swipe_right {
        state.drag_x = 0.0;
    }
}

pub fn use_gesture(
    on_swipe_left: impl FnMut() + 'static,
    on_swipe_right: impl FnMut() + 'static,
    on_double_click: impl FnMut() + 'static,
) -> (GestureHandlers, Signal<GestureState>) {
    let mut gesture_state = use_signal(GestureState::default);
    let on_swipe_left = Rc::new(RefCell::new(on_swipe_left));
    let on_swipe_right = Rc::new(RefCell::new(on_swipe_right));
    let on_double_click = Rc::new(RefCell::new(on_double_click));

    let performance = window().and_then(|w| w.performance());

    let handlers = GestureHandlers {
        on_touch_start: EventHandler::new(move |evt: Event<TouchData>| {
            if let Some(touch) = evt.data.as_ref().touches().first() {
                gesture_state.with_mut(|s| {
                    s.start_x = touch.client_coordinates().x;
                    s.start_y = touch.client_coordinates().y;
                    s.last_x = touch.client_coordinates().x;
                    s.last_y = touch.client_coordinates().y;
                    s.drag_x = 0.0;
                    s.is_swiping = true;
                    s.is_vertical_scroll_locked = false;
                });
            }
        }),
        on_touch_move: EventHandler::new(move |evt: Event<TouchData>| {
            if let Some(touch) = evt.data.as_ref().touches().first() {
                gesture_state.with_mut(|s| {
                    if s.is_swiping && !s.is_vertical_scroll_locked {
                        let distance_x = touch.client_coordinates().x - s.start_x;
                        let distance_y = touch.client_coordinates().y - s.start_y;

                        if distance_y.abs() > distance_x.abs() {
                            s.is_vertical_scroll_locked = true;
                            s.drag_x = 0.0;
                        } else {
                            s.last_x = touch.client_coordinates().x;
                            s.last_y = touch.client_coordinates().y;
                            s.drag_x = distance_x;
                        }
                    }
                });
            }
        }),
        on_touch_end: {
            let on_swipe_left = on_swipe_left.clone();
            let on_swipe_right = on_swipe_right.clone();
            let on_double_click = on_double_click.clone();
            let performance = performance.clone();
            EventHandler::new(move |_| {
                gesture_state.with_mut(|state| {
                    handle_gesture_end(
                        state,
                        &performance,
                        &on_swipe_left,
                        &on_swipe_right,
                        &on_double_click,
                    );
                    if let Some(p) = &performance {
                        state.last_touch_end_time = p.now();
                    }
                });
            })
        },
        on_mouse_down: {
            let performance = performance.clone();
            EventHandler::new(move |evt: Event<MouseData>| {
                if let Some(p) = &performance {
                    let now = p.now();
                    if now - gesture_state.with(|s| s.last_touch_end_time) < 500.0 {
                        return; // Ignore emulated mouse event
                    }
                }
                gesture_state.with_mut(|s| {
                    s.start_x = evt.data.client_coordinates().x;
                    s.start_y = evt.data.client_coordinates().y;
                    s.last_x = evt.data.client_coordinates().x;
                    s.last_y = evt.data.client_coordinates().y;
                    s.drag_x = 0.0;
                    s.is_swiping = true;
                    s.is_vertical_scroll_locked = false;
                });
            })
        },
        on_mouse_move: EventHandler::new(move |evt: Event<MouseData>| {
            gesture_state.with_mut(|s| {
                if s.is_swiping && !s.is_vertical_scroll_locked {
                    let distance_x = evt.data.client_coordinates().x - s.start_x;
                    let distance_y = evt.data.client_coordinates().y - s.start_y;

                    if distance_y.abs() > distance_x.abs() {
                        s.is_vertical_scroll_locked = true;
                        s.drag_x = 0.0;
                    } else {
                        s.last_x = evt.data.client_coordinates().x;
                        s.last_y = evt.data.client_coordinates().y;
                        s.drag_x = distance_x;
                    }
                }
            });
        }),
        on_mouse_up: {
            let on_swipe_left = on_swipe_left.clone();
            let on_swipe_right = on_swipe_right.clone();
            let on_double_click = on_double_click.clone();
            let performance = performance.clone();
            EventHandler::new(move |_| {
                gesture_state.with_mut(|state| {
                    handle_gesture_end(
                        state,
                        &performance,
                        &on_swipe_left,
                        &on_swipe_right,
                        &on_double_click,
                    );
                });
            })
        },
        on_mouse_leave: {
            EventHandler::new(move |_| {
                gesture_state.with_mut(|s| {
                    if s.is_swiping {
                        s.is_swiping = false;
                        s.drag_x = 0.0;
                    }
                });
            })
        },
    };

    (handlers, gesture_state)
}