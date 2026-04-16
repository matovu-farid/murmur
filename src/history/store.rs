use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionEntry {
    pub id: i64,
    pub timestamp: String,
    pub raw_text: String,
    pub cleaned_text: String,
    pub duration_secs: f32,
}

pub struct HistoryStore {
    conn: Connection,
}

impl HistoryStore {
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp    TEXT NOT NULL,
                raw_text     TEXT NOT NULL,
                cleaned_text TEXT NOT NULL,
                duration_secs REAL NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn new_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp    TEXT NOT NULL,
                raw_text     TEXT NOT NULL,
                cleaned_text TEXT NOT NULL,
                duration_secs REAL NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn insert(
        &self,
        raw_text: &str,
        cleaned_text: &str,
        duration_secs: f32,
    ) -> SqlResult<i64> {
        let timestamp = chrono::Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO transcriptions (timestamp, raw_text, cleaned_text, duration_secs) VALUES (?1, ?2, ?3, ?4)",
            params![timestamp, raw_text, cleaned_text, duration_secs],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all(&self) -> SqlResult<Vec<TranscriptionEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, raw_text, cleaned_text, duration_secs FROM transcriptions ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TranscriptionEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                raw_text: row.get(2)?,
                cleaned_text: row.get(3)?,
                duration_secs: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn search(&self, query: &str) -> SqlResult<Vec<TranscriptionEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, raw_text, cleaned_text, duration_secs FROM transcriptions \
             WHERE raw_text LIKE ?1 OR cleaned_text LIKE ?1 ORDER BY id DESC",
        )?;
        let pattern = format!("%{}%", query);
        let rows = stmt.query_map(params![pattern], |row| {
            Ok(TranscriptionEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                raw_text: row.get(2)?,
                cleaned_text: row.get(3)?,
                duration_secs: row.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete(&self, id: i64) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM transcriptions WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear(&self) -> SqlResult<()> {
        self.conn.execute("DELETE FROM transcriptions", [])?;
        Ok(())
    }

    pub fn export_json(&self) -> SqlResult<String> {
        let entries = self.get_all()?;
        Ok(serde_json::to_string_pretty(&entries).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_retrieve() {
        let store = HistoryStore::new_in_memory().unwrap();
        let id = store.insert("hello world", "Hello, world!", 1.5).unwrap();
        assert!(id > 0);
        let entries = store.get_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].raw_text, "hello world");
        assert_eq!(entries[0].cleaned_text, "Hello, world!");
    }

    #[test]
    fn test_search() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("hello world", "Hello, world!", 1.0).unwrap();
        store
            .insert("goodbye world", "Goodbye, world!", 2.0)
            .unwrap();
        let results = store.search("hello").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_delete() {
        let store = HistoryStore::new_in_memory().unwrap();
        let id = store.insert("test", "Test", 1.0).unwrap();
        store.delete(id).unwrap();
        assert!(store.get_all().unwrap().is_empty());
    }

    #[test]
    fn test_clear() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("one", "One", 1.0).unwrap();
        store.insert("two", "Two", 2.0).unwrap();
        store.clear().unwrap();
        assert!(store.get_all().unwrap().is_empty());
    }

    #[test]
    fn test_export_json() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("test", "Test", 1.0).unwrap();
        let json = store.export_json().unwrap();
        assert!(json.contains("test"));
    }

    #[test]
    fn test_order_is_newest_first() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("first", "First", 1.0).unwrap();
        store.insert("second", "Second", 2.0).unwrap();
        let entries = store.get_all().unwrap();
        assert_eq!(entries[0].raw_text, "second");
    }

    #[test]
    fn test_insert_multiple_and_retrieve() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("one", "One", 1.0).unwrap();
        store.insert("two", "Two", 2.0).unwrap();
        store.insert("three", "Three", 3.0).unwrap();
        let entries = store.get_all().unwrap();
        assert_eq!(entries.len(), 3);
        // Newest first
        assert_eq!(entries[0].raw_text, "three");
        assert_eq!(entries[1].raw_text, "two");
        assert_eq!(entries[2].raw_text, "one");
    }

    #[test]
    fn test_search_no_results() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("hello world", "Hello, world!", 1.0).unwrap();
        let results = store.search("nonexistent").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_case_sensitivity() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("Hello World", "Hello World", 1.0).unwrap();
        // SQLite LIKE is case-insensitive for ASCII by default
        let results = store.search("hello").unwrap();
        assert_eq!(results.len(), 1);
        let results_upper = store.search("HELLO").unwrap();
        assert_eq!(results_upper.len(), 1);
    }

    #[test]
    fn test_delete_nonexistent_id() {
        let store = HistoryStore::new_in_memory().unwrap();
        // Deleting a non-existent id should not error
        let result = store.delete(9999);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_json_valid_array() {
        let store = HistoryStore::new_in_memory().unwrap();
        store.insert("alpha", "Alpha", 1.0).unwrap();
        store.insert("beta", "Beta", 2.0).unwrap();
        let json = store.export_json().unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 2);
        // Each entry should have the expected fields
        assert!(parsed[0].get("raw_text").is_some());
        assert!(parsed[0].get("cleaned_text").is_some());
        assert!(parsed[0].get("timestamp").is_some());
        assert!(parsed[0].get("duration_secs").is_some());
    }
}
