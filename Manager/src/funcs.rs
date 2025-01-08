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
        format!("pkill -u {}", user),
    ];

    for command in commands {
        run_command(command)?;
    }

    conn.execute(
        "DELETE FROM users WHERE login_user = ?1",
        [user],
    )?;

    Ok("removed".to_string())
}



pub fn generate_test(time: usize, conn: &Connection) -> Result<String, Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    let n = rng.gen_range(1000..=9999);

    let user = format!("test{}", n);
    let pass = format!("test{}", n);

    let commands = [
        format!("/usr/sbin/useradd -M -N -s /bin/false {} -e $(date -d \"+{} minutes\" +%Y-%m-%dT%H:%M:%S)", user, time),
        format!("(echo {}; echo {}) | passwd {}", pass, pass, user),
        format!("echo \"/opt/rustymanager/manager --remove-user {}\" | at \"now + {} minute\"", user, time),
    ];

    for command in commands {
        run_command(command)?;
    }

    conn.execute(
        "INSERT INTO users (login_type, login_user, login_pass, login_limit, login_expiry) VALUES (?1, ?2, ?3, ?4, ?5)",
        ("test", &user, &pass, 1, minutes_to_expire_date(time)?.to_string()),
    )?;

    Ok(format!("user: {} | pass: {} | limit: {} | minutes remaining: {}", user, pass, 1, time))
}


pub fn change_validity(user: &str, days: usize, of_menu: bool, conn: &Connection) -> Result<String, Box<dyn Error>> {
    if !of_menu {
        if !user_already_exists(user)? {
            return Ok("user does not exist".to_string());
        }
    }
    run_command(format!("sudo chage -E $(date -d \"+{} days\" +%Y-%m-%d) {}", days, user))?;
    let new_expiry_date = days_to_expire_date(days)?;
    conn.execute(
        "UPDATE users SET login_expiry = ?1 WHERE login_user = ?2",
        (&new_expiry_date, user),
    )?;

    Ok(format!("changed | new expire date: {}", new_expiry_date))
}

pub fn change_limit(user: &str, limit: usize, of_menu: bool,  conn: &Connection) -> Result<String, Box<dyn Error>> {
    if !of_menu {
        if !user_already_exists(user)? {
            return Ok("user does not exist".to_string());
        }
    }
    conn.execute(
        "UPDATE users SET login_limit = ?1 WHERE login_user = ?2",
        (limit, user),
    )?;

    Ok(format!("changed | new limit: {}", limit))
}

pub fn change_pass(user: &str, pass: &str, of_menu: bool, conn: &Connection) -> Result<String, Box<dyn Error>> {
    if !of_menu {
        if !user_already_exists(user)? {
            return Ok("user does not exist".to_string());
        }
    }

    let commands = [
        format!("(echo {}; echo {}) | passwd {}", pass, pass, user),
        format!("pkill -u {}", user)
    ];

    for command in commands {
       run_command(command)?;
    }
    conn.execute(
        "UPDATE users SET login_pass = ?1 WHERE login_user = ?2",
        (pass, user),
    )?;
    Ok(format!("changed | new pass: {}", pass))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    login_type: String,
    pub(crate) user: String,
    pub(crate) pass: String,
    pub(crate) limit: String,
    pub(crate)  expiry: String,
}

pub fn userdata(user: &str, conn: &Connection) -> Result<String, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT login_type, login_user, login_pass, login_limit, login_expiry FROM users WHERE login_user = ?1")?;
    let user = stmt.query_row([user], |row| {
        Ok(User {
            login_type: row.get(0)?,
            user: row.get(1)?,
            pass: row.get(2)?,
            limit: row.get(3)?,
            expiry: row.get(4)?,
        })
    }).unwrap_or_else(|_| User {
        login_type: String::from("not found"),
        user: String::from("not found"),
        pass: String::from("not found"),
        limit: String::from("not found"),
        expiry: String::from("not found"),
    });
    Ok(serde_json::to_string_pretty(&user)?)
}

pub fn users_report_json(conn: &Connection) -> Result<String, Box<dyn Error>> {
    Ok(serde_json::to_string_pretty(&users_report_vec(conn)?)?)
}

pub fn users_report_vec(conn: &Connection) -> Result<Vec<User>, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT login_type, login_user, login_pass, login_limit, login_expiry FROM users")?;
    let user_iter = stmt.query_map([], |row| {
        Ok(User {
            login_type: row.get(0)?,
            user: row.get(1)?,
            pass: row.get(2)?,
            limit: row.get(3)?,
            expiry: row.get(4)?,
        })
    })?;

    Ok(user_iter.filter_map(Result::ok).collect())
}

pub fn expired_report_json(conn: &Connection) -> Result<String, Box<dyn Error>> {
    let expired_users = expired_report_vec(conn)?;
    Ok(serde_json::to_string_pretty(&expired_users)?)
}

pub fn expired_report_vec(conn: &Connection) -> Result<Vec<User>, Box<dyn Error>> {
    let all_users = users_report_vec(conn)?;
    Ok(expired_users(all_users)?)
}

fn expired_users(users: Vec<User>) -> Result<Vec<User>, Box<dyn Error>> {
    let mut vec_expired_users: Vec<User> = Vec::new();
    for user in &users {
        if user.login_type == "user" {
            let now = Local::now();
            if let Ok(expiry) = DateTime::parse_from_str(&user.expiry, "%Y-%m-%d %H:%M:%S%.3f %z") {
                if now > expiry {
                    vec_expired_users.push(user.clone());
                }
            }
        }
    }
   Ok(vec_expired_users)
}

pub fn user_already_exists(user: &str) -> Result<bool, Box<dyn Error>> {
    let exec = Command::new("bash")
        .arg("-c")
        .arg(format!("getent passwd {}", user))
        .output()?;

    if exec.status.success() {
        if !exec.stdout.is_empty() {
            return Ok(true)
        }
    }
    Ok(false)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connections {
    pub(crate) proxy: RustyProxy,
    pub(crate) stunnel: Stunnel,
    pub(crate) badvpn: BadVpn,
    pub(crate) checkuser: CheckUser,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustyProxy {
    pub(crate) ports: Option<Vec<u16>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stunnel {
    pub(crate) ports: Option<Vec<u16>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BadVpn {
    pub(crate) ports: Option<Vec<u16>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckUser {
    pub(crate) ports: Option<Vec<u16>>,
}





pub fn get_connections(conn: &Connection) -> Result<Connections, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT proxy_ports, stunnel_ports, badvpn_ports, checkuser_ports FROM connections LIMIT 1")?;

    let connection: Option<(Option<String>, Option<String>, Option<String>, Option<String>)> = stmt.query_row([], |row| {
        Ok((
            row.get(0).ok(),
            row.get(1).ok(),
            row.get(2).ok(),
            row.get(3).ok(),
        ))
    }).optional()?;

    match connection {
        Some((proxy_ports, stunnel_ports, badvpn_ports, checkuser_ports)) => {
            Ok(Connections {
                proxy: RustyProxy {
                    ports: Option::from(proxy_ports.map(|ports| {
                        ports.split('|').filter_map(|p| p.parse::<u16>().ok()).collect()
                    }).unwrap_or_else(|| Vec::new())),
                },
                stunnel: Stunnel {
                    ports: Option::from(stunnel_ports.map(|ports| {
                        ports.split('|').filter_map(|p| p.parse::<u16>().ok()).collect()
                    }).unwrap_or_else(|| Vec::new())),
                },
                badvpn: BadVpn {
                    ports: Option::from(badvpn_ports.map(|ports| {
                        ports.split('|').filter_map(|p| p.parse::<u16>().ok()).collect()
                    }).unwrap_or_else(|| Vec::new())),
                },
                checkuser: CheckUser {
                    ports: Option::from(checkuser_ports.map(|ports| {
                        ports.split('|').filter_map(|p| p.parse::<u16>().ok()).collect()
                    }).unwrap_or_else(|| Vec::new())),
                }
            })
        },
        None => Ok(Connections {
            proxy: RustyProxy {
                ports: Some(Vec::new()),
            },
            stunnel: Stunnel {
                ports: Some(Vec::new()),
            },
            badvpn: BadVpn {
                ports: Some(Vec::new()),
            },
            checkuser: CheckUser {
                ports: Some(Vec::new()),
            },
        })
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OnlineUser {
    pub(crate) user: String,
    pub(crate) connected: String,
    pub(crate) limit: String,
}
pub fn online_report_json(conn: &Connection) -> Result<String, Box<dyn Error>> {
    Ok(serde_json::to_string_pretty(&online_report(conn)?)?)
}
pub fn online_report(conn: &Connection) -> Result<Vec<OnlineUser>, Box<dyn Error>> {
    let output = run_command_and_get_output("ps -e -o user= -o cmd= | grep '[s]shd: ' | grep -v 'sshd: root@'")?;

    let mut online_users: Vec<OnlineUser> = Vec::new();
    let connections = String::from_utf8_lossy(output.as_ref());
    let mut user_connections: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for line in connections.lines() {
        let user = line.split_whitespace().next().unwrap_or("");
        if user != "root" && !user.is_empty() {
            *user_connections.entry(user).or_insert(0) += 1;
        }
    }

    for (user, count) in user_connections.iter() {
        let mut stmt = conn.prepare("SELECT login_type, login_user, login_pass, login_limit, login_expiry FROM users WHERE login_user = ?1")?;
        let db_user: User = stmt.query_row([user], |row| {
            Ok(User {
                login_type: row.get(0)?,
                user: row.get(1)?,
                pass: row.get(2)?,
                limit: row.get(3)?,
                expiry: row.get(4)?,
            })
        }).unwrap_or_else(|_| User {
            login_type: String::new(),
            user: String::new(),
            pass: String::new(),
            limit: String::from("0"),
            expiry: String::new(),
        });

        online_users.push(OnlineUser {
            user: user.to_string(),
            connected: count.to_string(),
            limit: db_user.limit,
        });
    }
    Ok(online_users)
}


pub fn is_port_avaliable(port: usize) -> Result<bool, bool> {
    match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(_) => {
            Ok(true)
        },
        Err(_) => {
            Ok(false)
        }
    }
}
pub fn enable_proxy_port(port: String, status: String) -> Result<(), Box<dyn Error>> {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn proxy --enable-port {} --status {}", port, status))?;
    Ok(())
}
pub fn disable_proxy_port(port: String) -> Result<(), Box<dyn Error>>{
    run_command(format!("/opt/rustymanager/connectionsmanager --conn proxy --disable-port {}", port))?;
     Ok(())
}
pub fn enable_stunnel_port(port: String, ipv6: bool) -> Result<(), Box<dyn Error>>{
    if ipv6 {
         run_command(format!("/opt/rustymanager/connectionsmanager --conn stunnel --enable-port {} --ipv6 true", port))?;
    } else {
       run_command(format!("/opt/rustymanager/connectionsmanager --conn stunnel --enable-port {}", port))?;
    }
     Ok(())
}
pub fn disable_stunnel_port(port: String) -> Result<(), Box<dyn Error>> {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn stunnel --disable-port {}", port))?;
     Ok(())
}

pub fn enable_badvpn_port(port: String) -> Result<(), Box<dyn Error>> {
   run_command(format!("/opt/rustymanager/connectionsmanager --conn badvpn --enable-port {}", port))?;
    Ok(())
}

pub fn disable_badvpn_port(port: String) -> Result<(), Box<dyn Error>> {
   run_command(format!("/opt/rustymanager/connectionsmanager --conn badvpn --disable-port {}", port))?;
    Ok(())
}

pub fn enable_checkuser_port(port: String) -> Result<(), Box<dyn Error>> {
    run_command(format!("/opt/rustymanager/connectionsmanager --conn checkuser --enable-port {}", port))?;
      Ok(())
}

pub fn disable_checkuser_port(port: String) -> Result<(), Box<dyn Error>>{
    run_command(format!("/opt/rustymanager/connectionsmanager --conn checkuser --disable-port {}", port))?;
    Ok(())
}

pub fn journald_status() -> Result<bool, Box<dyn Error>> {
    let output = run_command_and_get_output("systemctl is-active systemd-journald.service")?;
     Ok(output == "active")
}

pub fn enable_journald() -> Result<(), Box<dyn Error>>{
    let commands = [
        "systemctl start --now systemd-journald.service systemd-journald-audit.socket systemd-journald-dev-log.socket systemd-journald.socket".to_string(),
        "systemctl enable --now systemd-journald.service systemd-journald-audit.socket systemd-journald-dev-log.socket systemd-journald.socket".to_string()
    ];
    for command in commands {
         run_command(command)?;
    }
    Ok(())
}

pub fn disable_journald() -> Result<(), Box<dyn Error>> {
    let commands = [
        "systemctl stop --now systemd-journald.service systemd-journald-audit.socket systemd-journald-dev-log.socket systemd-journald.socket".to_string(),
        "systemctl disable --now systemd-journald.service systemd-journald-audit.socket systemd-journald-dev-log.socket systemd-journald.socket".to_string()
    ];
    for command in commands {
         run_command(command)?;
    }
   Ok(())
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SpeedTestData {
    pub(crate) r#type: String,
    pub(crate) timestamp: String,
    pub(crate) ping: Ping,
    pub(crate)  download: Transfer,
    pub(crate)  upload: Transfer,
    #[serde(rename = "packetLoss")]
    pub(crate)  packet_loss: f64,
    pub(crate)   isp: String,
    pub(crate)   interface: NetworkInterface,
    pub(crate)  server: Server,
    pub(crate)  result: ResultInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ping {
    pub(crate) jitter: f64,
    pub(crate)  latency: f64,
    pub(crate)  low: f64,
    pub(crate)  high: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transfer {
    pub(crate)   bandwidth: u64,
    pub(crate)   bytes: u64,
    pub(crate)   elapsed: u64,
    pub(crate)   latency: Latency,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Latency {
    pub(crate)   iqm: f64,
    pub(crate)   low: f64,
    pub(crate)   high: f64,
    pub(crate)   jitter: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkInterface {
    #[serde(rename = "internalIp")]
    pub(crate)  internal_ip: String,
    pub(crate)  name: String,
    #[serde(rename = "macAddr")]
    pub(crate)  mac_addr: String,
    #[serde(rename = "isVpn")]
    pub(crate)    is_vpn: bool,
    #[serde(rename = "externalIp")]
    pub(crate)  external_ip: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Server {
    pub(crate)  id: u64,
    pub(crate)  host: String,
    pub(crate)  port: u16,
    pub(crate)  name: String,
    pub(crate)  location: String,
    pub(crate)   country: String,
    pub(crate)  ip: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResultInfo {
    pub(crate)   id: String,
    pub(crate)    url: String,
    pub(crate)  persisted: bool,
}
pub fn speedtest_data() -> Result<SpeedTestData, Box<dyn Error>> {
    let json = run_command_and_get_output("speedtest --accept-license --accept-gdpr -f json")?;
    Ok(serde_json::from_str(&json)?)
}



#[derive(Debug)]
pub struct Service {
    pub(crate) name: String,
    pub(crate) ports: Vec<u16>,
}

pub fn get_services() -> Result<Vec<Service>, Box<dyn Error>> {
    let command = "netstat -tulnp | awk '/LISTEN/ {split($4, a, \":\"); split($7, b, \"/\"); gsub(\":\", \"\", b[2]); if (!seen[b[2] a[length(a)]]++) ports[b[2]] = ports[b[2]] \" \" a[length(a)]} END {for (service in ports) print service, ports[service]}' | sort -u";

    let output_str = run_command_and_get_output(command)?;
    let mut services_map: HashMap<String, Vec<u16>> = HashMap::new();

    for line in output_str.lines() {
        let mut parts = line.split_whitespace();
        if let Some(service_name) = parts.next() {
            let service_name = service_name.to_string();
            let ports: Vec<u16> = parts
                .filter_map(|port_str| port_str.parse::<u16>().ok())
                .collect();

            services_map.insert(service_name, ports);
        }
    }

    Ok(services_map
        .into_iter()
        .map(|(name, ports)| Service { name, ports })
        .collect())
}



fn run_command(command: String) -> Result<(), Box<dyn Error>> {
    let exec = Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()?;

    if !exec.status.success() {
        return Err(format!("Command failed: {}", command).into());
    }
    Ok(())
}

pub fn run_command_and_get_output(command: &str) -> Result<String, Box<dyn Error>> {
    let exec = Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()?;

    if !exec.status.success() {
        return Ok(String::new());
    }

    let output = std::str::from_utf8(&exec.stdout)?;
    Ok(output.trim().to_string())
}


fn days_to_expire_date(days: usize) -> Result<String, Box<dyn Error>> {
    let now: DateTime<Local> = Local::now();
    let expiry_date = now + Duration::days(days as i64);
    Ok(expiry_date.to_string())
}

fn minutes_to_expire_date(minutes: usize) -> Result<String, Box<dyn Error>> {
    let now: DateTime<Local> = Local::now();
    let expiry_date = now + Duration::minutes(minutes as i64);
    Ok(expiry_date.to_string())
}
