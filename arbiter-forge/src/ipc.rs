use crate::{ArbiterForge, LogEntry, LOG_MODEL};
use arbiter_core::protocol::{
    ForgeCommand, LogEntry as WireLogEntry, PIPE_COMMAND, PIPE_TELEMETRY,
};
use slint::{Color, Model};
use std::time::Duration;

pub async fn send_command(cmd: &ForgeCommand) {
    use tokio::net::windows::named_pipe::ClientOptions;

    let result = async {
        use futures::SinkExt;
        use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};

        let client = ClientOptions::new().open(PIPE_COMMAND)?;
        let mut framed = FramedWrite::new(client, LengthDelimitedCodec::new());
        let bin = rmp_serde::to_vec_named(cmd).map_err(std::io::Error::other)?;
        framed.send(bytes::Bytes::from(bin)).await?;
        Ok::<(), std::io::Error>(())
    }
    .await;

    if let Err(e) = result {
        tracing::error!(%e, "Forge: IPC send failed");
        LOG_MODEL.with(|m| {
            m.push(LogEntry {
                time: chrono::Local::now().format("%H:%M:%S").to_string().into(),
                tag: "IPC".into(),
                tag_color: Color::from_rgb_u8(244, 63, 94),
                msg: format!("IPC Failure: {e}").into(),
                decree_id: "".into(),
            });
        });
    } else {
        tracing::info!("Forge: Command sent successfully");
        LOG_MODEL.with(|m| {
            m.push(LogEntry {
                time: chrono::Local::now().format("%H:%M:%S").to_string().into(),
                tag: "FORGE".into(),
                tag_color: Color::from_rgb_u8(34, 197, 94),
                msg: "Command committed to engine successfully.".into(),
                decree_id: "".into(),
            });
        });
    }
}

pub async fn app_is_available(wait_for: Duration) -> bool {
    use tokio::net::windows::named_pipe::ClientOptions;
    let deadline = std::time::Instant::now() + wait_for;
    loop {
        if ClientOptions::new().open(PIPE_COMMAND).is_ok() {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

pub fn spawn_telemetry_listener(ui_handle: slint::Weak<ArbiterForge>) {
    tokio::spawn(async move {
        use futures::StreamExt;
        use tokio::net::windows::named_pipe::ClientOptions;
        use tokio::time::timeout;
        use tokio_util::codec::FramedRead;

        let watchdog_duration = Duration::from_secs(2);

        loop {
            let client = if let Ok(c) = ClientOptions::new().open(PIPE_TELEMETRY) {
                c
            } else {
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            };

            let mut framed =
                FramedRead::new(client, tokio_util::codec::LengthDelimitedCodec::new());

            loop {
                match timeout(watchdog_duration, framed.next()).await {
                    Ok(Some(Ok(bytes))) => match rmp_serde::from_slice::<WireLogEntry>(&bytes) {
                        Ok(core_entry) => {
                            if core_entry.tag == "VIGIL" && core_entry.message.contains("Heartbeat")
                            {
                                continue;
                            }

                            let ui_copy = ui_handle.clone();

                            let tag_color = match core_entry.tag.as_str() {
                                "VIGIL" | "Vigil-fs" | "Vigil-keys" => {
                                    Color::from_rgb_u8(99, 102, 241)
                                }
                                "ATLAS" | "Atlas" => Color::from_rgb_u8(245, 158, 11),
                                "RUNNER" | "Runner" => Color::from_rgb_u8(16, 185, 129),
                                "BATON" | "Baton" => Color::from_rgb_u8(244, 63, 94),
                                "ERROR" => Color::from_rgb_u8(244, 63, 94),
                                "WARN" => Color::from_rgb_u8(245, 158, 11),
                                _ => Color::from_rgb_u8(161, 161, 170),
                            };

                            let time_str = if core_entry.time.is_empty() {
                                chrono::Local::now().format("%H:%M:%S").to_string()
                            } else if let Ok(dt) =
                                chrono::DateTime::parse_from_rfc3339(&core_entry.time)
                            {
                                dt.with_timezone(&chrono::Local)
                                    .format("%H:%M:%S")
                                    .to_string()
                            } else {
                                core_entry.time.clone()
                            };

                            let is_runner =
                                core_entry.tag == "RUNNER" || core_entry.tag == "Runner";
                            let is_done = is_runner
                                && (core_entry.message.contains("complete")
                                    || core_entry.message.contains("finished")
                                    || core_entry.message.contains("error")
                                    || core_entry.message.contains("aborted"));

                            let should_sync = core_entry.tag == "ATLAS"
                                && (core_entry.message.contains("registered and saved")
                                    || core_entry.message.contains("removed and saved"));
                            let should_sync_signet = core_entry.tag == "VIGIL"
                                && core_entry.message.contains("Conservatory Wards updated");

                            let entry = LogEntry {
                                time: time_str.into(),
                                tag: core_entry.tag.into(),
                                msg: core_entry.message.into(),
                                tag_color,
                                decree_id: core_entry.decree_id.unwrap_or_default().into(),
                            };

                            let _ = ui_copy.upgrade_in_event_loop(move |ui| {
                                LOG_MODEL.with(|m| {
                                    m.push(entry);
                                    if m.row_count() > 50 {
                                        m.remove(0);
                                    }
                                });
                                if should_sync {
                                    crate::sync_ledger_to_ui();
                                }
                                if should_sync_signet {
                                    crate::sync_signet_to_ui(&ui);
                                }
                                if is_runner {
                                    ui.set_engine_running(!is_done);
                                }
                                ui.invoke_scroll_logs_to_bottom();
                            });
                        }
                        Err(e) => {
                            tracing::error!("Forge: failed to parse telemetry MessagePack: {e}");
                        }
                    },
                    Ok(Some(Err(e))) => {
                        tracing::error!("Forge: telemetry pipe error: {e}");
                        break;
                    }
                    Ok(None) => {
                        tracing::warn!("Forge: telemetry pipe closed by engine.");
                        break;
                    }
                    Err(_) => {
                        tracing::error!("Forge: Watchdog expired (2s silence). Engine likely terminated. Requesting graceful exit.");
                        let _ = ui_handle.upgrade_in_event_loop(|ui| {
                            ui.invoke_request_close();
                        });
                        return;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}
