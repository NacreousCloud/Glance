#![cfg(target_os = "windows")]

use super::{NotiEvent, NotificationSource, Publisher};
use parking_lot::Mutex;
use std::sync::Arc;
use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    UI::Notifications::{
        KnownNotificationBindings,
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        UserNotificationChangedEventArgs,
    },
};

pub struct WindowsNotiSource {
    token: Arc<Mutex<Option<EventRegistrationToken>>>,
}

impl WindowsNotiSource {
    pub fn new() -> Self {
        Self {
            token: Arc::new(Mutex::new(None)),
        }
    }

    pub fn access_status() -> windows::core::Result<UserNotificationListenerAccessStatus> {
        let listener = UserNotificationListener::Current()?;
        listener.RequestAccessAsync()?.get()
    }
}

impl Default for WindowsNotiSource {
    fn default() -> Self {
        Self::new()
    }
}

impl NotificationSource for WindowsNotiSource {
    fn start(&self, publish: Publisher) -> anyhow::Result<()> {
        let listener = UserNotificationListener::Current()?;
        let status = listener.RequestAccessAsync()?.get()?;
        if status != UserNotificationListenerAccessStatus::Allowed {
            anyhow::bail!("Notification listener access denied: {status:?}");
        }

        let publish: Arc<Publisher> = Arc::new(publish);
        let publish_handler = publish.clone();
        let handler =
            TypedEventHandler::<UserNotificationListener, UserNotificationChangedEventArgs>::new(
                move |_sender, args| {
                    let Some(args) = args.as_ref() else {
                        return Ok(());
                    };
                    let id = args.UserNotificationId()?;
                    let listener = UserNotificationListener::Current()?;
                    let notif = listener.GetNotification(id)?;
                    let app_info = notif.AppInfo()?;
                    let app_id = app_info.AppUserModelId()?.to_string_lossy();
                    let app_name = app_info.DisplayInfo()?.DisplayName()?.to_string_lossy();
                    let binding = notif
                        .Notification()?
                        .Visual()?
                        .GetBinding(&KnownNotificationBindings::ToastGeneric()?)?;
                    let texts = binding.GetTextElements()?;
                    let mut title = String::new();
                    let mut body = String::new();
                    if let Ok(t) = texts.GetAt(0) {
                        title = t.Text()?.to_string_lossy();
                    }
                    if let Ok(b) = texts.GetAt(1) {
                        body = b.Text()?.to_string_lossy();
                    }
                    (publish_handler)(NotiEvent::now(app_id, app_name, title, body));
                    Ok(())
                },
            );

        let token = listener.NotificationChanged(&handler)?;
        *self.token.lock() = Some(token);
        Ok(())
    }

    fn stop(&self) {
        if let Some(token) = self.token.lock().take() {
            if let Ok(listener) = UserNotificationListener::Current() {
                let _ = listener.RemoveNotificationChanged(token);
            }
        }
    }
}
