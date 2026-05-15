use crate::filter::ArbiterFilter;
use tokio::sync::mpsc;
use tracing::{info, trace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceSignal {
    MouseInput,
    KeyboardInput,
}

pub fn spawn_monitor(
    tx: mpsc::Sender<PresenceSignal>,
    filter: ArbiterFilter,
) -> std::thread::JoinHandle<()> {
    info!("Presence monitor spawned");

    std::thread::spawn(move || {
        use rdev::{listen, Event, EventType};

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Presence: tokio runtime failed");

        let callback = move |event: Event| {
            if filter.is_inhibited() {
                return;
            }

            let signal = match event.event_type {
                EventType::MouseMove { .. }
                | EventType::ButtonPress(_)
                | EventType::ButtonRelease(_)
                | EventType::Wheel { .. } => Some(PresenceSignal::MouseInput),
                EventType::KeyPress(_) | EventType::KeyRelease(_) => {
                    Some(PresenceSignal::KeyboardInput)
                }
            };

            if let Some(sig) = signal {
                trace!(?sig, "Presence: input detected");
                let tx = tx.clone();
                rt.block_on(async move {
                    let _ = tx.send(sig).await;
                });
            }
        };

        if let Err(e) = listen(callback) {
            tracing::warn!(?e, "Presence monitor exited with error");
        }

        info!("Presence monitor thread exiting");
    })
}
