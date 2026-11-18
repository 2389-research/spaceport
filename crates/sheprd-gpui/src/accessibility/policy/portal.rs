// ABOUTME: Reads the desktop portal's standard appearance preferences on Linux.
// ABOUTME: Keeps missing keys distinct from explicit normal motion or contrast.
use super::Preferences;
use std::collections::HashMap;
use zbus::zvariant::OwnedValue;

type Settings = HashMap<String, HashMap<String, OwnedValue>>;
const APPEARANCE: &str = "org.freedesktop.appearance";

/// The deadline includes connection/authentication, not just the method reply.
#[cfg(any(target_os = "linux", test))]
pub(super) async fn with_deadline(
    read: impl Future<Output = zbus::Result<Preferences>>,
    deadline: impl Future<Output = ()>,
) -> Preferences {
    futures_lite::future::race(
        async {
            match read.await {
                Ok(preferences) => preferences,
                Err(error) => {
                    tracing::debug!(%error, "Desktop accessibility preferences unavailable");
                    Preferences::default()
                }
            }
        },
        async {
            deadline.await;
            Preferences::default()
        },
    )
    .await
}

#[cfg(target_os = "linux")]

pub(super) async fn read() -> zbus::Result<Preferences> {
    let connection = zbus::Connection::session().await?;
    read_from(&connection).await
}

async fn read_from(connection: &zbus::Connection) -> zbus::Result<Preferences> {
    let proxy = zbus::Proxy::new(
        connection,
        "org.freedesktop.portal.Desktop",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.portal.Settings",
    )
    .await?;
    let namespaces: &[&str] = &[APPEARANCE];
    let settings: Settings = proxy.call("ReadAll", &(namespaces,)).await?;
    Ok(decode(&settings))
}

fn decode(settings: &Settings) -> Preferences {
    let value = |name| {
        settings
            .get(APPEARANCE)?
            .get(name)
            .and_then(|value| u32::try_from(value).ok())
            // The portal specifies unknown values as no preference.
            .map(|value| value == 1)
    };
    Preferences {
        high_contrast: value("contrast"),
        reduce_motion: value("reduced-motion"),
    }
}

#[cfg(test)]
mod tests;
