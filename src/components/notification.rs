use dioxus::prelude::*;
use std::time::Duration;

#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub enum NotificationLevel {
    Success,
    Error,
    Info,
    Warning,
    Progress,
}

#[derive(Clone, PartialEq)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub progress: Option<f32>,
    pub id: u64,
}

#[component]
pub fn NotificationDisplay(notifications: Signal<Vec<Notification>>) -> Element {
    rsx! {
        div {
            class: "notification-container",
            for notif in notifications.read().iter() {
                NotificationItem { notification: notif.clone() }
            }
        }
    }
}

#[component]
fn NotificationItem(notification: Notification) -> Element {
    let mut is_visible = use_signal(|| true);
    
    let notification_clone = notification.clone();
    use_effect(move || {
        if notification_clone.level != NotificationLevel::Progress {
            let mut is_visible_clone = is_visible;
            spawn(async move {
                gloo_timers::future::sleep(Duration::from_secs(5)).await;
                *is_visible_clone.write() = false;
            });
        }
    });

    if !is_visible() {
        return rsx! { };
    }

    rsx! {
        div {
            key: "{notification.id}",
            class: "notification-item {get_notification_class(notification.level.clone())}",
            div {
                class: "notification-content",
                div {
                    class: "notification-message",
                    "{notification.message}"
                }
                if let Some(progress) = notification.progress {
                    progress {
                        class: "progress-bar",
                        max: "100",
                        value: "{progress * 100.0}",
                    }
                }
            }
            button {
                class: "notification-close",
                onclick: move |_| *is_visible.write() = false,
                "×"
            }
        }
    }
}

fn get_notification_class(level: NotificationLevel) -> &'static str {
    match level {
        NotificationLevel::Success => "notification-success",
        NotificationLevel::Error => "notification-error",
        NotificationLevel::Info => "notification-info",
        NotificationLevel::Warning => "notification-warning",
        NotificationLevel::Progress => "notification-progress",
    }
}

pub fn use_notification() -> (Signal<Vec<Notification>>, NotificationManager) {
    let notifications = use_signal::<Vec<Notification>>(Vec::new);
    let next_id = use_signal(|| 0u64);

    let manager = NotificationManager {
        notifications,
        next_id,
    };

    (notifications, manager)
}

#[derive(Clone)]
pub struct NotificationManager {
    notifications: Signal<Vec<Notification>>,
    next_id: Signal<u64>,
}

impl NotificationManager {
    fn schedule_remove(&self, id: u64, delay: Duration) {
        let mut manager = self.clone();
        spawn(async move {
            gloo_timers::future::sleep(delay).await;
            manager.remove(id);
        });
    }

    #[allow(dead_code)]
    pub fn show(&mut self, message: String, level: NotificationLevel) {
        let id = *self.next_id.read();
        *self.next_id.write() = id + 1;
        let should_auto_remove = level != NotificationLevel::Progress;

        let notification = Notification {
            message,
            level,
            progress: None,
            id,
        };
        
        self.notifications.write().push(notification);
        if should_auto_remove {
            self.schedule_remove(id, Duration::from_secs(1));
        }
    }

    pub fn show_progress(&mut self, message: String, progress: f32) -> ProgressHandle {
        let id = *self.next_id.read();
        *self.next_id.write() = id + 1;
        
        let notification = Notification {
            message,
            level: NotificationLevel::Progress,
            progress: Some(progress),
            id,
        };
        
        self.notifications.write().push(notification.clone());
        
        ProgressHandle {
            manager: self.clone(),
            id,
        }
    }

    pub fn update_progress(&mut self, id: u64, progress: f32) {
        if let Some(notification) = self.notifications.write().iter_mut().find(|n| n.id == id) {
            notification.progress = Some(progress);
        }
    }

    pub fn complete_progress(&mut self, id: u64, message: String, success: bool) {
        if let Some(notification) = self.notifications.write().iter_mut().find(|n| n.id == id) {
            notification.message = message;
            notification.level = if success {
                NotificationLevel::Success
            } else {
                NotificationLevel::Error
            };
            notification.progress = None;
        }
        self.schedule_remove(id, Duration::from_secs(1));
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, id: u64) {
        self.notifications.write().retain(|n| n.id != id);
    }
}

#[derive(Clone)]
pub struct ProgressHandle {
    manager: NotificationManager,
    id: u64,
}

impl ProgressHandle {
    pub fn update(&mut self, progress: f32) {
        self.manager.update_progress(self.id, progress);
    }

    pub fn complete(mut self, message: String, success: bool) {
        self.manager.complete_progress(self.id, message, success);
    }
}
