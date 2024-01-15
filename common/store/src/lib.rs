use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{prelude::*, utils::Instant};
use rusqlite::OptionalExtension;
use rusqlite_migration::{Migrations, M};
use serde::{de::DeserializeOwned, Serialize};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalStore>();
    }
}

#[derive(Resource)]
pub struct GlobalStore {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

pub struct Entry<'a, T> {
    store: &'a Mutex<rusqlite::Connection>,
    key: &'static str,

    _phantom: PhantomData<T>,
}

impl<'a, T: Serialize + DeserializeOwned> Entry<'a, T> {
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
            Some(ron::from_str(&raw_value).expect("Cannot deserialize"));

        let ms = now.elapsed().as_millis();
        if ms > 1 {
            warn!("Entry::get({}) took {ms}ms", self.key);
        }

        value
    }

    pub fn set(&self, value: T) {
        let now = Instant::now();

        let raw_value = ron::to_string(&value).expect("Cannot serialize");

        {
            let conn = self.store.lock().unwrap();
            conn.execute(
                "INSERT INTO kv (key, value) VALUES (?, ?)
                ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                [&self.key, &raw_value.as_str()],
            )
            .expect("Cannot insert into SQLite");
        }

        let ms = now.elapsed().as_millis();
        if ms > 1 {
            warn!("Entry::set({}) took {ms}ms", self.key);
        }
    }

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

pub use apartment::ApartmentStore;
mod apartment {
    use std::time::Duration;

    use super::*;

    pub trait ApartmentStore {
        fn position_on_load(&self) -> Entry<'_, Vec2>;

        fn walk_to_onload(&self) -> Entry<'_, Vec2>;

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

    pub trait DowntownStore {
        fn position_on_load(&self) -> Entry<'_, Vec2>;

        fn walk_to_onload(&self) -> Entry<'_, Vec2>;

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

impl GlobalStore {
    pub fn new() -> Self {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();

        migrate(&mut conn);

        Self {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    fn entry<T>(&self, key: &'static str) -> Entry<'_, T> {
        Entry::new(&self.conn, key)
    }
}

impl Default for GlobalStore {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> Entry<'a, T> {
    fn new(store: &'a Mutex<rusqlite::Connection>, key: &'static str) -> Self {
        Self {
            store,
            key,
            _phantom: PhantomData,
        }
    }
}
fn migrate(conn: &mut rusqlite::Connection) {
    let migrations = Migrations::new(vec![M::up(
        "CREATE TABLE kv (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );",
    )]);

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

    fn new_conn() -> Arc<Mutex<rusqlite::Connection>> {
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();

        migrate(&mut conn);

        Arc::new(Mutex::new(conn))
    }
}
