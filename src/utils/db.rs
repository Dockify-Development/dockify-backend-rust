/*
    This source file is a part of Dockify
    Dockify is licensed under the Server Side Public License (SSPL), Version 1.
    Find the LICENSE file in the root of this repository for more details.
*/

use bollard::container::Config;
use rusqlite::{params, Connection, Result};
use serde::Serialize;
use std::{fs::File, path::Path};

pub async fn insert_container(
    id: &String,
    username: String,
    name: String,
    config: Config<String>,
    port: u16,
) -> Result<usize, rusqlite::Error> {
    let conn = Connection::open("./dockify.db")?;
    let config = match config.host_config {
        Some(x) => x,
        None => return Ok(0),
    };
    match conn.execute(
        "INSERT INTO containers (id, username, name, memory, memory_swap, cpu_cores, cpu_shares, port) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![id, username, name, config.memory, config.memory_swap, config.nano_cpus, config.cpu_shares, port],
    ) {
        Ok(updated) => {
            println!("{} rows were updated", updated);
            return Ok(updated);
        }
        Err(err) => {
            println!("update failed: {}", err);
            return Ok(0);
        }
    }
}

pub async fn create_db() {
    if Path::new("./dockify.db").exists() {
        return;
    }
    println!("Creating Dockify database file...");
    match File::create("dockify.db") {
        Ok(_) => {
            let conn = Connection::open("./dockify.db").unwrap();
            let stmts = [
                "CREATE TABLE IF NOT EXISTS containers (
                    id TEXT PRIMARY KEY UNIQUE NOT NULL,
                    username TEXT NOT NULL,
                    name TEXT NOT NULL,
                    memory INTEGER NOT NULL,
                    memory_swap INTEGER NOT NULL,
                    cpu_cores INTEGER NOT NULL,
                    cpu_shares INTEGER NOT NULL,
                    port INTEGER NOT NULL
                )",
                "CREATE TABLE IF NOT EXISTS users (
                    email TEXT PRIMARY KEY UNIQUE NOT NULL,
                    username TEXT UNIQUE NOT NULL,
                    dusername TEXT UNIQUE NOT NULL,
                    hash TEXT NOT NULL,
                    verified INTEGER NOT NULL,
                    max INTEGER,
                    admin INTEGER
                )",
                "CREATE TABLE IF NOT EXISTS ip_logs (
                    username TEXT PRIMARY KEY UNIQUE NOT NULL,
                    ip TEXT NOT NULL
                )",
                "CREATE TABLE IF NOT EXISTS verification_codes (
                    verification_code TEXT PRIMARY KEY UNIQUE NOT NULL,
                    username TEXT NOT NULL
                )",
                "CREATE TABLE IF NOT EXISTS credits (
                    username TEXT PRIMARY KEY UNIQUE NOT NULL,
                    credits INTEGER NOT NULL
                )",
            ];
            for stmt in stmts {
                match conn.execute(stmt, []) {
                    Ok(_) => {
                        println!("Created new table")
                    }
                    Err(err) => {
                        tracing::error!("Error while creating table: {}", err)
                    }
                }
            }
        }
        Err(err) => {
            tracing::error!("An error occurred while creating Dockify DataBase: {}", err);
        }
    }
}
pub fn check_exists(
    row: impl Into<String>,
    column: impl Into<String>,
    table: impl Into<String>,
) -> Result<bool> {
    let conn = Connection::open("./dockify.db")?;
    let query = format!(
        "SELECT 1 FROM {} WHERE {} = ?1 LIMIT 1",
        table.into(),
        column.into()
    );

    let exists = conn
        .query_row(&query, params![row.into()], |_| Ok(()))
        .is_ok();

    Ok(exists)
}
pub fn insert_code(username: &str, code: &str) -> Result<()> {
    Connection::open("./dockify.db")?.execute(
        "INSERT INTO verification_codes (username, verification_code) VALUES (?1, ?2)",
        params![username, code],
    )?;
    Ok(())
}
pub fn remove_code(code: &str) -> Result<()> {
    Connection::open("./dockify.db")?.execute(
        "DELETE FROM verification_codes WHERE verification_code = ?1",
        params![code],
    )?;
    Ok(())
}

pub fn insert_user(email: &str, username: &str, hash: &str, verified: bool) -> Result<()> {
    Connection::open("./dockify.db")?.execute(
        "INSERT INTO users (email, username, hash, verified, dusername) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![email.to_lowercase(), username.to_lowercase(), hash, verified, username],
    )?;
    Ok(())
}

pub fn verify_user(username: &str) -> Result<(), rusqlite::Error> {
    let conn = Connection::open("./dockify.db")?;
    conn.execute(
        "UPDATE users 
         SET verified = ?1
         WHERE username = ?2",
        params![true, username],
    )?;
    Ok(())
}

pub fn get_user_info(identifier: &str) -> Result<(String, i32, String, String)> {
    let conn = Connection::open("./dockify.db")?;

    let mut stmt = conn.prepare(
        "SELECT hash, verified, username, email FROM users WHERE username = ?1 OR email = ?1",
    )?;

    let mut rows = stmt.query(params![identifier])?;

    if let Some(row) = rows.next()? {
        let hash: String = row.get(0)?;
        let verified: i32 = row.get(1)?;
        let username: String = row.get(2)?;
        let email: String = row.get(3)?;
        Ok((hash, verified, username, email))
    } else {
        Err(rusqlite::Error::QueryReturnedNoRows)
    }
}
pub fn is_admin(username: impl Into<String>) -> Result<bool> {
    let conn = Connection::open("./dockify.db")?;

    let mut stmt = conn.prepare("SELECT admin FROM users WHERE username = ?1")?;

    let admin: Option<i64> = stmt
        .query_row(params![username.into()], |row| row.get(0))
        .ok();

    Ok(match admin {
        Some(1) => true,
        Some(_) => false,
        None => false,
    })
}

pub fn count_containers_by_username(id: &str) -> Result<i32> {
    let conn = Connection::open("./dockify.db")?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM containers WHERE username = ?1")?;
    let count: i32 = stmt.query_row(params![id], |row| row.get(0))?;
    Ok(count)
}
#[allow(dead_code)]
#[derive(Serialize)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub memory: i64,
    pub memory_swap: i64,
    pub cpu_shares: i64,
    pub cpu_cores: i64,
    pub port: i64,
}
pub fn get_user_containers(username: &str) -> Result<Vec<Container>> {
    let conn = Connection::open("./dockify.db")?;

    let mut stmt = conn.prepare(
        "SELECT id, name, memory, memory_swap, cpu_shares, cpu_cores, port FROM containers WHERE username = ?1"
    )?;

    let mut rows = stmt.query(params![username])?;
    let mut containers: Vec<Container> = Vec::new();
    if let Some(row) = rows.next()? {
        let container = Container {
            id: row.get(0)?,
            name: row.get(1)?,
            memory: row.get(2)?,
            memory_swap: row.get(3)?,
            cpu_shares: row.get(4)?,
            cpu_cores: row.get(5)?,
            port: row.get(6)?,
        };
        containers.insert(containers.len(), container);
    } else {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(containers)
}
pub fn get_user_credits(username: &str) -> Result<i64> {
    let conn = Connection::open("./dockify.db")?;
    let mut stmt = conn.prepare("SELECT credits FROM credits WHERE username = ?1")?;

    let credits: Option<i64> = stmt.query_row(params![username], |row| row.get(0)).ok();

    match credits {
        Some(credits) => Ok(credits),
        None => {
            set_user_credits(username, 0)?;
            Ok(0)
        }
    }
}
pub fn set_user_credits(username: &str, credits: i64) -> Result<()> {
    let conn = Connection::open("./dockify.db")?;
    conn.execute(
        "INSERT INTO credits (username, credits)
         VALUES (?1, ?2)
         ON CONFLICT(username) DO UPDATE SET credits = excluded.credits",
        params![username, credits],
    )?;
    Ok(())
}
