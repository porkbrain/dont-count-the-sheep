//! A way to communicate between scenes and across time (save/load).

#![deny(missing_docs)]

use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
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

pub use dialog::DialogStore;
mod dialog {
    use super::*;

    /// Store anything that's related to dialogs.
    /// History and choices, etc.
    pub trait DialogStore {
        /// Get the last dialog entry's namespace and node name.
        fn get_last_dialog<T: From<String>>(&self) -> Option<(T, String)>;

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

        /// Next time dialog is started with this NPC, the player will get
        /// an option to start from this dialog.
        ///
        /// Idempotent.
        fn add_dialog_to_npc(
            &self,
            npc: impl Display,
            namespace: impl Display,
        ) -> &Self;

        /// Remove the dialog from the NPC's list of dialogs.
        ///
        /// Idempotent.
        fn remove_dialog_from_npc(
            &self,
            npc: impl Display,
            namespace: impl Display,
        );

        /// List all the dialogs that the NPC has.
        fn list_dialogs_for_npc<T: From<String>>(
            &self,
            npc: impl Display,
        ) -> Vec<T>;

        /// Get the last dialog entry's type path.
        fn was_this_the_last_dialog<T: Eq + From<String>>(
            &self,
            (expected_namespace, expected_name): (T, impl Display),
        ) -> bool {
            self.get_last_dialog::<T>()
                .is_some_and(|(namespace, name)| {
                    namespace == expected_namespace
                        && name == expected_name.to_string()
                })
        }
    }

    impl DialogStore for GlobalStore {
        fn get_last_dialog<T: From<String>>(&self) -> Option<(T, String)> {
            let conn = self.conn.lock().unwrap();

            let now = Instant::now();
            let value = conn
                .query_row(
                    "SELECT namespace, node_name FROM dialog_nodes_transitioned_to \
                    ORDER BY id DESC LIMIT 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
                .expect("Cannot query SQLite");
            let ms = now.elapsed().as_millis();
            if ms > 1 {
                warn!("get_last_dialog took {ms}ms");
            }

            value.map(|(namespace, node_name): (String, String)| {
                (namespace.into(), node_name)
            })
        }

        fn insert_dialog(
            &self,
            (namespace, node_name): (impl Display, impl Display),
        ) {
            let conn = self.conn.lock().unwrap();

            let now = Instant::now();
            conn.execute(
                "INSERT INTO dialog_nodes_transitioned_to (namespace, node_name) \
                VALUES (:namespace, :node_name)",
                named_params! {
                    ":namespace": namespace.to_string(),
                    ":node_name": node_name.to_string(),
                },
            )
            .expect("Cannot insert into SQLite");

            let ms = now.elapsed().as_millis();
            if ms > 1 {
                warn!("insert_dialog took {ms}ms");
            }
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

        fn add_dialog_to_npc(
            &self,
            npc: impl Display,
            namespace: impl Display,
        ) -> &Self {
            let conn = self.conn.lock().unwrap();

            let now = Instant::now();
            conn.execute(
                "INSERT OR IGNORE INTO npc_dialogs \
                (npc, namespace) VALUES (:npc, :namespace)",
                named_params! {
                    ":npc": npc.to_string(),
                    ":namespace": namespace.to_string(),
                },
            )
            .expect("Cannot insert into SQLite");

            let ms = now.elapsed().as_millis();
            if ms > 1 {
                warn!("add_dialog_to_npc took {ms}ms");
            }

            self
        }

        fn remove_dialog_from_npc(
            &self,
            npc: impl Display,
            namespace: impl Display,
        ) {
            let conn = self.conn.lock().unwrap();

            let now = Instant::now();
            conn.execute(
                "DELETE FROM npc_dialogs WHERE npc = :npc AND namespace = :namespace",
                named_params! {
                    ":npc": npc.to_string(),
                    ":namespace": namespace.to_string(),
                },
            )
            .expect("Cannot delete from SQLite");

            let ms = now.elapsed().as_millis();
            if ms > 1 {
                warn!("remove_dialog_from_npc took {ms}ms");
            }
        }

        fn list_dialogs_for_npc<T: From<String>>(
            &self,
            npc: impl Display,
        ) -> Vec<T> {
            let conn = self.conn.lock().unwrap();

            let now = Instant::now();
            let mut stmt = conn
                .prepare("SELECT namespace FROM npc_dialogs WHERE npc = :npc")
                .expect("Cannot prepare SQLite");
            let rows = stmt
                .query_map(
                    named_params! {
                        ":npc": npc.to_string(),
                    },
                    |row| row.get(0),
                )
                .expect("Cannot query SQLite");

            let ms = now.elapsed().as_millis();
            if ms > 1 {
                warn!("list_dialogs_for_npc took {ms}ms");
            }

            rows.map(|row| String::into(row.expect("Cannot get row")))
                .collect()
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
        // generic key value table
        M::up(
            "CREATE TABLE kv (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        ),
        // dialogs storage
        M::up(
            "CREATE TABLE dialog_nodes_transitioned_to (
                id INTEGER PRIMARY KEY,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                namespace TEXT NOT NULL,
                node_name TEXT NOT NULL
            );",
        ),
        M::up(
            "CREATE INDEX idx_dialog_nodes_transitioned_to_namespace_node_name \
            ON dialog_nodes_transitioned_to (namespace, node_name);",
        ),
        M::up(
            "CREATE TABLE npc_dialogs (
                npc TEXT NOT NULL,
                namespace TEXT NOT NULL
            );",
        ),
        M::up(
            "CREATE INDEX idx_npc_dialogs_npc_namespace \
            ON npc_dialogs (npc, namespace);",
        ),
        // what have the player already seen using the inspect ability
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
        assert!(store.was_this_the_last_dialog::<String>((
            "ok/dialog.toml".to_owned(),
            "node1"
        )));
        assert!(!store.was_this_the_last_dialog::<String>((
            "ok/dialog.toml".to_owned(),
            "node2"
        )));
        assert!(!store.was_this_the_last_dialog::<String>((
            "no/dialog.toml".to_owned(),
            "node1"
        )));

        store.insert_dialog(("ok/dialog.toml", "node2"));
        assert!(store.was_this_the_last_dialog::<String>((
            "ok/dialog.toml".to_owned(),
            "node2"
        )));
        assert!(!store.was_this_the_last_dialog::<String>((
            "ok/dialog.toml".to_owned(),
            "node1"
        )));
    }

    fn new_conn() -> Arc<Mutex<rusqlite::Connection>> {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();

        migrate(&mut conn);

        Arc::new(Mutex::new(conn))
    }
}
