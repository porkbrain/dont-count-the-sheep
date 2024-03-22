//! A way to communicate between scenes and across time (save/load).
#![deny(missing_docs)]

use std::{
    borrow::{Borrow, Cow},
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{prelude::*, utils::Instant};
use rusqlite::{named_params, OptionalExtension};
use rusqlite_migration::{Migrations, M};
use serde::{de::DeserializeOwned, Serialize};

/// Inits the store.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalStore>();
    }
}

/// SQLite database under the hood.
#[derive(Resource)]
pub struct GlobalStore {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

/// A key-value entry that you can read, write and remove.
pub struct Entry<'a, T> {
    store: &'a Mutex<rusqlite::Connection>,
    key: Cow<'static, str>,

    _phantom: PhantomData<T>,
}

impl<'a, T: Serialize + DeserializeOwned> Entry<'a, T> {
    /// Get the deserialized value.
    pub fn get(&self) -> Option<T> {
        let now = Instant::now();

        let raw_value: String = {
            let conn = self.store.lock().unwrap();
            conn.query_row(
                "SELECT value FROM kv WHERE key = ?",
                [&self.key],
                |row| row.get(0),
            )
            .optional()
            .expect("Cannot query SQLite")?
        };

        let value =
            Some(serde_json::from_str(&raw_value).expect("Cannot deserialize"));

        let ms = now.elapsed().as_millis();
        if ms > 1 {
            warn!("Entry::get({}) took {ms}ms", self.key);
        }

        value
    }

    /// Write a value over the key that is serializable.
    pub fn set(&self, value: T) {
        let now = Instant::now();

        let raw_value =
            serde_json::to_string(&value).expect("Cannot serialize");

        {
            let conn = self.store.lock().unwrap();
            conn.execute(
                "INSERT INTO kv (key, value) VALUES (?, ?)
                ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                [&self.key.borrow(), &raw_value.as_str()],
            )
            .expect("Cannot insert into SQLite");
        }

        let ms = now.elapsed().as_millis();
        if ms > 1 {
            warn!("Entry::set({}) took {ms}ms", self.key);
        }
    }
}

impl<'a, T> Entry<'a, T> {
    /// Remove the entry from db.
    pub fn remove(&self) {
        let now = Instant::now();

        {
            let conn = self.store.lock().unwrap();
            conn.execute("DELETE FROM kv WHERE key = ?", [&self.key])
                .expect("Cannot delete from SQLite");
        }

        let ms = now.elapsed().as_millis();
        if ms > 1 {
            warn!("Entry::remove({}) took {ms}ms", self.key);
        }
    }
}

pub use inspect_ability::InspectAbilityStore;
mod inspect_ability {
    use super::*;

    /// Store anything that's related to inspect ability.
    pub trait InspectAbilityStore {
        /// Mark a given inspect label as seen by the player.
        ///
        /// Idempotent.
        fn mark_as_seen(&self, label: &str);
    }

    impl InspectAbilityStore for GlobalStore {
        fn mark_as_seen(&self, label: &str) {
            let now = Instant::now();

            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO
                discovered_with_inspect_ability (label) VALUES (:label)
                ON CONFLICT DO NOTHING",
                named_params! {
                    ":label": label,
                },
            )
            .expect("Cannot insert into SQLite");

            let ms = now.elapsed().as_millis();
            if ms > 1 {
                warn!("mark_as_seen took {ms}ms");
            }
        }
    }
}

pub use apartment::ApartmentStore;
mod apartment {
    use std::time::Duration;

    use super::*;

    /// Apartment store data.
    pub trait ApartmentStore {
        /// When the player loads the apartment, where should they be?
        fn position_on_load(&self) -> Entry<'_, Vec2>;

        /// When the player loads the apartment, where should they walk to?
        /// This creates a nice effect of the player walking to the apartment.
        fn walk_to_onload(&self) -> Entry<'_, Vec2>;

        /// When the player loads the apartment, how fast should they walk?
        fn step_time_onload(&self) -> Entry<'_, Duration>;
    }

    impl ApartmentStore for GlobalStore {
        fn position_on_load(&self) -> Entry<'_, Vec2> {
            self.entry("apartment.position_on_load")
        }

        fn walk_to_onload(&self) -> Entry<'_, Vec2> {
            self.entry("apartment.walk_towards_onload")
        }

        fn step_time_onload(&self) -> Entry<'_, Duration> {
            self.entry("apartment.step_time_onload")
        }
    }
}

pub use downtown::DowntownStore;
mod downtown {
    use std::time::Duration;

    use super::*;

    /// Downtown store data.
    pub trait DowntownStore {
        /// When the player loads the downtown, where should they be?
        fn position_on_load(&self) -> Entry<'_, Vec2>;

        /// When the player loads the downtown, where should they walk to?
        /// This creates a nice effect of the player walking to the downtown.
        fn walk_to_onload(&self) -> Entry<'_, Vec2>;

        /// When the player loads the downtown, how fast should they walk?
        fn step_time_onload(&self) -> Entry<'_, Duration>;
    }

    impl DowntownStore for GlobalStore {
        fn position_on_load(&self) -> Entry<'_, Vec2> {
            self.entry("downtown.position_on_load")
        }

        fn walk_to_onload(&self) -> Entry<'_, Vec2> {
            self.entry("downtown.walk_towards_onload")
        }

        fn step_time_onload(&self) -> Entry<'_, Duration> {
            self.entry("apartment.step_time_onload")
        }
    }
}

pub use dialog::DialogStore;
mod dialog {
    use std::fmt::Display;

    use super::*;

    /// Store anything that's related to dialogs.
    /// History and choices, etc.
    pub trait DialogStore {
        /// Get the last dialog entry's type path.
        fn was_this_the_last_dialog(
            &self,
            namespace_and_name: (impl Display, impl Display),
        ) -> bool;

        /// New dialog entry.
        fn insert_dialog(
            &self,
            namespace_and_name: (impl Display, impl Display),
        );

        /// Access guard state using a unique guard kind id and a unique node
        /// name.
        ///
        /// Unique node name is going to include the dialog file and the node
        /// name within that file.
        /// Unique guard kind id might be the enum variant name.
        fn guard_state(
            &self,
            guard_kind: impl Display,
            namespace_and_name: (impl Display, impl Display),
        ) -> Entry<'_, serde_json::Value>;
    }

    impl DialogStore for GlobalStore {
        fn was_this_the_last_dialog(
            &self,
            (namespace, node_name): (impl Display, impl Display),
        ) -> bool {
            let conn = self.conn.lock().unwrap();

            let value = conn
                .query_row(
                    "SELECT namespace, node_name FROM dialogs \
                    ORDER BY id DESC LIMIT 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .expect("Cannot query SQLite");

            value
                .map(|(last_namespace, last_node_name): (String, String)| {
                    last_namespace == namespace.to_string()
                        && last_node_name == node_name.to_string()
                })
                .unwrap_or(false)
        }

        fn insert_dialog(
            &self,
            (namespace, node_name): (impl Display, impl Display),
        ) {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "INSERT INTO dialogs (namespace, node_name) \
                VALUES (:namespace, :node_name)",
                named_params! {
                    ":namespace": namespace.to_string(),
                    ":node_name": node_name.to_string(),
                },
            )
            .expect("Cannot insert into SQLite");
        }

        fn guard_state(
            &self,
            guard_kind: impl Display,
            (namespace, node_name): (impl Display, impl Display),
        ) -> Entry<'_, serde_json::Value> {
            self.entry(format!(
                "dialog.guard_state.{namespace}.{guard_kind}.{node_name}"
            ))
        }
    }
}

impl GlobalStore {
    /// Create a new store.
    pub fn new() -> Self {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();

        migrate(&mut conn);

        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    fn entry<T>(&self, key: impl Into<Cow<'static, str>>) -> Entry<'_, T> {
        Entry::new(&self.conn, key)
    }
}

impl Default for GlobalStore {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> Entry<'a, T> {
    fn new(
        store: &'a Mutex<rusqlite::Connection>,
        key: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            store,
            key: key.into(),
            _phantom: PhantomData,
        }
    }
}

fn migrate(conn: &mut rusqlite::Connection) {
    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE kv (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        ),
        M::up(
            "CREATE TABLE dialogs (
                id INTEGER PRIMARY KEY,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                namespace TEXT NOT NULL,
                node_name TEXT NOT NULL
            );",
        ),
        M::up(
            "CREATE INDEX idx_dialogs_namespace_node_name \
            ON dialogs (namespace, node_name);",
        ),
        M::up(
            "CREATE TABLE discovered_with_inspect_ability (
                label TEXT PRIMARY KEY
            );",
        ),
    ]);

    migrations.to_latest(conn).unwrap();
}

#[cfg(test)]
mod tests {
    use bevy::math::vec2;

    use super::*;

    #[test]
    fn it_serializes_i32() {
        let conn = new_conn();
        let store = GlobalStore { conn };

        let entry = store.entry::<i32>("test");
        assert_eq!(entry.get(), None);

        entry.set(42);

        assert_eq!(entry.get(), Some(42));
    }

    #[test]
    fn it_serializes_vec2() {
        let conn = new_conn();
        let store = GlobalStore { conn };

        let entry = store.entry::<Vec2>("test");
        assert_eq!(entry.get(), None);

        entry.set(vec2(0.0, 1.0));

        assert_eq!(entry.get(), Some(vec2(0.0, 1.0)));
    }

    #[test]
    fn it_inserts_dialogs() {
        let conn = new_conn();
        let store = GlobalStore { conn };

        store.insert_dialog(("ok/dialog.toml", "node1"));
        assert!(store.was_this_the_last_dialog(("ok/dialog.toml", "node1")));
        assert!(!store.was_this_the_last_dialog(("ok/dialog.toml", "node2")));
        assert!(!store.was_this_the_last_dialog(("no/dialog.toml", "node1")));

        store.insert_dialog(("ok/dialog.toml", "node2"));
        assert!(store.was_this_the_last_dialog(("ok/dialog.toml", "node2")));
        assert!(!store.was_this_the_last_dialog(("ok/dialog.toml", "node1")));
    }

    fn new_conn() -> Arc<Mutex<rusqlite::Connection>> {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();

        migrate(&mut conn);

        Arc::new(Mutex::new(conn))
    }
}
