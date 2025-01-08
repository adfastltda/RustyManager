use std::{fs, io};
use std::io::Write;
use std::net::TcpListener;
use std::process::Command;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Connections {
    pub(crate) proxy: RustyProxy,
    pub(crate) stunnel: Stunnel,
    pub(crate) badvpn: BadVpn,
    pub(crate) checkuser: CheckUser,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RustyProxy {
    pub(crate) ports: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stunnel {
    pub(crate) ports: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BadVpn {
    pub(crate) ports: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CheckUser {
    pub(crate) ports: Option<String>,
}

pub fn is_port_available(port: usize) -> Result<bool, bool> {
    match TcpListener::bind(format!("0.0.0.0:{}", port)) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn add_proxy_port(port: usize, status: Option<String>) -> Result<(), io::Error> {
    let mut command = format!("/opt/rustymanager/rustyproxy --port {}", port);
    if let Some(status_str) = status {
        command = format!("{} --status \"{}\"", command, status_str);
    }

    let service_file_content = format!(r#"
[Unit]
Description=RustyProxy{}
After=network.target

[Service]
LimitNOFILE=infinity
LimitNPROC=infinity
LimitMEMLOCK=infinity
LimitSTACK=infinity
LimitCORE=infinity
LimitAS=infinity
LimitRSS=infinity
LimitCPU=infinity
LimitFSIZE=infinity
Type=simple
ExecStart={}
Restart=always

[Install]
WantedBy=multi-user.target
"#, port, command);

    let service_file_path = format!("/etc/systemd/system/rustyproxy{}.service", port);
    fs::write(&service_file_path, service_file_content)?;
    
    run_command("systemctl daemon-reload".to_string())?;
    run_command(format!("systemctl enable rustyproxy{}.service", port))?;
    run_command(format!("systemctl start rustyproxy{}.service", port))?;

    Ok(())
}
pub fn del_proxy_port(port: usize) -> Result<(), io::Error> {
    run_command(format!("systemctl disable rustyproxy{}.service", port))?;
    run_command(format!("systemctl stop rustyproxy{}.service", port))?;
    fs::remove_file(format!("/etc/systemd/system/rustyproxy{}.service", port))?;
    Ok(())
}

pub fn add_stunnel_port(port: usize, ipv6: bool) -> std::result::Result<(), io::Error> {
    let port_str = port.to_string();
    let prefix = if ipv6 { ":::" } else { "0.0.0.0:" };
    let command = format!("grep -qE '^(::|0\\.0\\.0\\.0:)?{port_str}$' /etc/stunnel/stunnel.conf || echo 'accept = {prefix}{port_str}' | sudo tee -a /etc/stunnel/stunnel.conf");
    run_command(command)?;

    if run_command("systemctl is-active --quiet stunnel4".to_string()).is_ok() {
        run_command("sudo systemctl restart stunnel4".to_string())?;
    } else {
        run_command("sudo systemctl start stunnel4".to_string())?;
    }

    Ok(())
}
pub fn del_stunnel_port(port: usize) -> std::result::Result<(), io::Error> {
    let port_str = port.to_string();
    run_command(format!("sed -i '/{port_str}/d' /etc/stunnel/stunnel.conf"))?;
    if run_command("grep -q 'accept' /etc/stunnel/stunnel.conf".to_string()).is_ok(){
        run_command("systemctl restart stunnel4.service".to_string())?;
    } else {
        run_command("sudo systemctl stop stunnel4".to_string())?;
    }
    Ok(())
}
pub fn add_badvpn_port(port: usize) -> std::result::Result<(), io::Error> {
    let service_file_content = format!(r#"
[Unit]
Description=BadVpn{}
After=network.target

[Service]
LimitNOFILE=infinity
LimitNPROC=infinity
LimitMEMLOCK=infinity
LimitSTACK=infinity
LimitCORE=infinity
LimitAS=infinity
LimitRSS=infinity
LimitCPU=infinity
LimitFSIZE=infinity
Restart=always
Type=simple
ExecStart=/opt/rustymanager/badvpn --listen-addr [::]:{} --max-clients 1000 --max-connections-for-client 1000 --client-socket-sndbuf 0 --udp-mtu 9000

[Install]
WantedBy=multi-user.target
"#, port, port);

    let service_file_path = format!("/etc/systemd/system/badvpn{}.service", port);

    fs::write(&service_file_path, service_file_content)?;

    run_command("systemctl daemon-reload".to_string())?;
    run_command(format!("systemctl enable badvpn{}.service", port))?;
    run_command(format!("systemctl start badvpn{}.service", port))?;

    Ok(())
}
pub fn del_badvpn_port(port: usize) -> std::result::Result<(), io::Error> {
    run_command(format!("systemctl disable badvpn{}.service", port))?;
    run_command(format!("systemctl stop badvpn{}.service", port))?;
    fs::remove_file(format!("/etc/systemd/system/badvpn{}.service", port))?;
    Ok(())
}
pub fn add_checkuser_port(port: usize) -> std::result::Result<(), io::Error> {
    let service_file_content = format!(r#"
[Unit]
Description=Checkuser{}
After=network.target

[Service]
LimitNOFILE=infinity
LimitNPROC=infinity
LimitMEMLOCK=infinity
LimitSTACK=infinity
LimitCORE=infinity
LimitAS=infinity
LimitRSS=infinity
LimitCPU=infinity
LimitFSIZE=infinity
Restart=always
Type=simple
ExecStart=/opt/rustymanager/checkuser --port {}

[Install]
WantedBy=multi-user.target
"#, port, port);

    let service_file_path = format!("/etc/systemd/system/checkuser{}.service", port);

    fs::write(&service_file_path, service_file_content)?;

    run_command("systemctl daemon-reload".to_string())?;
    run_command(format!("systemctl enable checkuser{}.service", port))?;
    run_command(format!("systemctl start checkuser{}.service", port))?;
    Ok(())
}
pub fn del_checkuser_port(port: usize) -> std::result::Result<(), io::Error> {
    run_command(format!("systemctl disable checkuser{}.service", port))?;
    run_command(format!("systemctl stop checkuser{}.service", port))?;
    fs::remove_file(format!("/etc/systemd/system/checkuser{}.service", port))?;
    Ok(())
}

pub fn add_proxy_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    add_port_to_db(sqlite_conn, "proxy_ports", port)
}

pub fn add_stunnel_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    add_port_to_db(sqlite_conn, "stunnel_ports", port)
}

pub fn add_badvpn_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
     add_port_to_db(sqlite_conn, "badvpn_ports", port)
}

pub fn add_checkuser_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    add_port_to_db(sqlite_conn, "checkuser_ports", port)
}

fn add_port_to_db(sqlite_conn: &Connection, column: &str, port: u16) -> Result<(), rusqlite::Error> {
     let mut stmt = sqlite_conn.prepare("SELECT * FROM connections LIMIT 1")?;
    let connections: Vec<Connections> = stmt.query_map(params![], |row| {
        Ok(Connections {
            proxy: RustyProxy {
                ports: row.get::<_, String>(1).ok(),
            },
            stunnel: Stunnel {
                ports: row.get::<_, String>(2).ok(),
            },
            badvpn: BadVpn {
                ports: row.get::<_, String>(3).ok(),
            },
            checkuser: CheckUser {
                ports: row.get::<_, String>(4).ok(),
            },
        })
    })?.collect::<Result<_, _>>()?;

    match connections.first() {
        Some(conn) => {
             let ports = match column {
                "proxy_ports" => &conn.proxy.ports,
                "stunnel_ports" => &conn.stunnel.ports,
                "badvpn_ports" => &conn.badvpn.ports,
                "checkuser_ports" => &conn.checkuser.ports,
                _ => &None
            };
            let mut ports_str = ports.clone().unwrap_or_default();

            if !ports_str.is_empty() {
               ports_str.push('|');
            }

           ports_str.push_str(&port.to_string());
           let query = format!("UPDATE connections SET {} = ? WHERE id = 1", column);
           sqlite_conn.execute(&query, params![ports_str])?;
           Ok(())
       },
       None => {
           let query = format!("INSERT INTO connections (proxy_ports, stunnel_ports, badvpn_ports, checkuser_ports) VALUES (NULL, NULL, NULL, NULL)");
           sqlite_conn.execute(&query, params![])?;
           let query = format!("UPDATE connections SET {} = ? WHERE id = 1", column);
           sqlite_conn.execute(&query, params![port.to_string()])?;
           Ok(())
       }
   }
}

pub fn del_proxy_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    del_port_from_db(sqlite_conn, "proxy_ports", port)
}

pub fn del_stunnel_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    del_port_from_db(sqlite_conn, "stunnel_ports", port)
}

pub fn del_badvpn_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
   del_port_from_db(sqlite_conn, "badvpn_ports", port)
}

pub fn del_checkuser_port_in_db(sqlite_conn: &Connection, port: u16) -> Result<(), rusqlite::Error> {
    del_port_from_db(sqlite_conn, "checkuser_ports", port)
}


fn del_port_from_db(sqlite_conn: &Connection, column: &str, port: u16) -> Result<(), rusqlite::Error> {
    let query = format!("SELECT {} FROM connections LIMIT 1", column);
    let mut stmt = sqlite_conn.prepare(&query)?;
    let connections: Vec<String> = stmt.query_map(params![], |row| {
        row.get::<_, String>(0)
    })?.collect::<Result<_, _>>()?;

    if let Some(existing_ports) = connections.first() {
        let mut ports_vec: Vec<String> = existing_ports.trim().split('|').map(String::from).collect();
        ports_vec.retain(|p| p != &port.to_string());
        let new_ports = ports_vec.join("|");
        let query = format!("UPDATE connections SET {} = ? WHERE id = 1", column);
        sqlite_conn.execute(&query, params![new_ports])?;
        Ok(())
    } else {
        Err(rusqlite::Error::UnwindingPanic)
    }
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
