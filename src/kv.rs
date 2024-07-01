use rusqlite::Connection;

pub struct Kv {
    connection: Connection,
}

impl Kv {
    pub fn new() -> Kv {
        let dir = xdg::BaseDirectories::with_prefix("waybar")
            .expect("No waybar directory found in XDG_CONFIG_DIRS");
        let path = dir.get_config_file("rice.db");
        let exists = path.exists();

        let connection = Connection::open(path).expect("failed to open the database");
        if !exists {
            connection
                .execute(
                    "CREATE TABLE dictionary (
                    key TEXT PRIMARY KEY,
                    bool BOOLEAN,
                    text TEXT,
                    i64 INTEGER,
                    f64 DECIMAL
                )",
                    [],
                )
                .expect("failed to create dictionary table");
        }

        Kv { connection }
    }

    pub fn put_bool(&self, key: &str, value: bool) -> () {
        self.connection
            .execute(
                "INSERT INTO dictionary (key, bool) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET bool = ?2",
                rusqlite::params![key, value],
            )
            .expect("failed to insert into the database");
    }

    pub fn put_string(&self, key: &str, value: &str) -> () {
        self.connection
            .execute(
                "INSERT INTO dictionary (key, text) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET text = ?2",
                rusqlite::params![key, value],
            )
            .expect("failed to insert into the database");
    }

    pub fn put_i64(&self, key: &str, value: i64) -> () {
        self.connection
            .execute(
                "INSERT INTO dictionary (key, i64) VALUES (?1, ?2) ON CONFLICT DO UPDATE SET i64 = ?2",
                rusqlite::params![key, value],
            )
            .expect("failed to insert into the database");
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT bool FROM dictionary WHERE key = ?1 LIMIT 1")
            .expect("failed to prepare the statement");

        stmt.query_row([key], |row| row.get(0)) as Result<bool, rusqlite::Error>
    }

    pub fn get_string(&self, key: &str) -> Result<String, rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT text FROM dictionary WHERE key = ?1 LIMIT 1")
            .expect("failed to prepare the statement");

        stmt.query_row([key], |row| row.get(0)) as Result<String, rusqlite::Error>
    }

    pub fn get_i64(&self, key: &str) -> Result<i64, rusqlite::Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT i64 FROM dictionary WHERE key = ?1 LIMIT 1")
            .expect("failed to prepare the statement");

        stmt.query_row([key], |row| row.get(0)) as Result<i64, rusqlite::Error>
    }
}
