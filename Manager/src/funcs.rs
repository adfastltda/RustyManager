use std::collections::HashMap;
use std::net::{TcpListener};
use chrono::{DateTime, Duration, Local};
use serde::{Deserialize, Serialize};
use std::process::{Command};
use rand::Rng;
use rusqlite::{Connection, OptionalExtension};
use std::error::Error;


pub fn create_user(user: &str, pass: &str, days: usize, limit: usize, of_menu: bool, conn: &Connection) -> Result<String, Box<dyn Error>> {
    if !of_menu {
        if user_already_exists(user)? {
            return Ok("user already exists".to_string());
        }
    }

    let commands = [
        format!("/usr/sbin/useradd -M -N -s /bin/false {} -e $(date -d \"+{} days\" +%Y-%m-%d\n)", user, days),
        format!("(echo {}; echo {}) | passwd {}", pass, pass, user)
    ];

    for command in commands {
        run_command(command)?;
    }

    conn.execute(
        "INSERT INTO users (login_type, login_user, login_pass, login_limit, login_expiry) VALUES (?1, ?2, ?3, ?4, ?5)",
        ("user", user, pass, limit, days_to_expire_date(days)?.to_string()),
    )?;

    Ok("created".to_string())
}


pub fn remove_user(user: &str, of_menu: bool, conn: &Connection) -> Result<String, Box<dyn Error>> {
    if !of_menu {
        if !user_already_exists(user)? {
            return Ok("user does not exist".to_string());
        }
    }

    let commands = [
        format!("userdel --force {}", user),
        format!("p
