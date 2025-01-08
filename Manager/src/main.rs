mod funcs;
mod text_funcs;

use std::{env, io, thread};
use std::io::BufRead;
use std::process::Command;
use std::time::Duration;
use chrono::DateTime;
use rusqlite::Connection;
use crate::text_funcs::{text_to_bold};
use crate::funcs::{create_user, change_limit, change_pass, change_validity, expired_report_json, expired_report_vec, generate_test, is_port_avaliable, remove_user, user_already_exists, users_report_json, users_report_vec, run_command_and_get_output, get_connections, enable_badvpn_port, disable_badvpn_port, enable_proxy_port, disable_proxy_port, enable_stunnel_port, disable_stunnel_port, online_report_json, online_report, userdata, speedtest_data, enable_checkuser_port, disable_checkuser_port, journald_status, disable_journald, enable_journald, get_services};

fn main() {
    let sqlite_conn = Connection::open("/opt/rustymanager/db").unwrap();
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        main_menu(&sqlite_conn);
    } else{
        match (&args[1]).as_str() {
            "--create-user" => handle_create_user(&args, &sqlite_conn),
            "--remove-user" => handle_remove_user(&args, &sqlite_conn),
            "--generate-test" => handle_generate_test(&args, &sqlite_conn),
            "--change-limit" => handle_change_limit(&args, &sqlite_conn),
            "--change-validity" => handle_change_validity(&args, &sqlite_conn),
            "--change-pass" => handle_change_pass(&args, &sqlite_conn),
            "--userdata" => handle_userdata(&args, &sqlite_conn),
            "--users-report" => handle_users_report(&sqlite_conn),
            "--expired-report" => handle_expired_report(&sqlite_conn),
            "--online-report" => handle_online_report(&sqlite_conn),
            "--help" => print_help(),
            _ => println!("função invalida selecionada")
        }
    }
}
fn handle_create_user(args: &Vec<String>, sqlite_conn: &Connection) {
     match args.len() {
         _i if 2 >= _i  => {
             println!("user empty");
             return;
         }
         _i if 3 >= _i  => {
             println!("pass empty");
             return;
         }
         _i if 4 >= _i => {
             println!("days empty");
             return;
         }
         _i if 5 >= _i => {
             println!("limit empty");
             return;
         }
         _ => {}
     }

     let days = &args[4];
     let limit = &args[5];
    match days.parse::<usize>() {
       Ok(..) => {}
       Err(..) => {
           println!("invalid digit found in days");
           return
       }
   }
    match limit.parse::<usize>() {
      Ok(..) => {}
       Err(..) => {
          println!("invalid digit found in limit");
          return
       }
   }
   let string = create_user(&args[2], &args[3], days.parse().unwrap(), limit.parse().unwrap(), false, sqlite_conn);
   match string {
        Ok(msg) => println!("{}", msg),
        Err(err) => println!("Error creating user: {}", err)
    }
}
fn handle_remove_user(args: &Vec<String>, sqlite_conn: &Connection) {
    if 2 >= args.len() {
         println!("user empty");
         return;
    }
    let string = remove_user(&args[2], false, sqlite_conn);
    match string {
         Ok(msg) => println!("{}", msg),
        Err(err) => println!("Error removing user: {}", err)
     }
}
fn handle_generate_test(args: &Vec<String>, sqlite_conn: &Connection) {
   if 2 >= args.len() {
        println!("minutes empty");
        return;
   }

   let days = &args[2];
    match days.parse::<usize>() {
        Ok(..) => {}
        Err(..) => {
           println!("invalid digit found in days");
           return
       }
    }
     let string = generate_test(days.parse().unwrap(), sqlite_conn);
    match string {
        Ok(msg) => println!("{}", msg),
         Err(err) => println!("Error generating test user: {}", err)
    }
}
fn handle_change_limit(args: &Vec<String>, sqlite_conn: &Connection) {
    match args.len() {
        _i if 2 >= _i  => {
            println!("user empty");
            return;
        }
        _i if 3 >= _i  => {
            println!("limit empty");
            return;
        }
        _ => {}
    }
   let limit = &args[3];
    match limit.parse::<usize>() {
      Ok(..) => {}
        Err(..) => {
           println!("invalid digit found in limit");
            return
        }
    }
  let string = change_limit(&args[2], limit.parse().unwrap(), false, sqlite_conn);
    match string {
       Ok(msg) => println!("{}", msg),
         Err(err) => println!("Error changing limit: {}", err)
   }
}
fn handle_change_validity(args: &Vec<String>, sqlite_conn: &Connection) {
    match args.len() {
        _i if 2 >= _i  => {
           println!("user empty");
           return;
        }
        _i if 3 >= _i  => {
            println!("days empty");
            return;
        }
       _ => {}
    }
   let days = &args[3];
    match days.parse::<usize>() {
       Ok(..) => {}
       Err(..) => {
            println!("invalid digit found in days");
            return
       }
    }
    let string = change_validity(&args[2], days.parse().unwrap(), false, sqlite_conn);
     match string {
        Ok(msg) => println!("{}", msg),
        Err(err) => println!("Error changing validity: {}", err)
   }
}
fn handle_change_pass(args: &Vec<String>, sqlite_conn: &Connection) {
    match args.len() {
        _i if 2 >= _i  => {
            println!("user empty");
           return;
        }
        _i if 3 >= _i  => {
            println!("pass empty");
            return;
        }
        _ => {}
    }
    let string = change_pass(&args[2], &args[3], false, sqlite_conn);
    match string {
        Ok(msg) => println!("{}", msg),
         Err(err) => println!("Error changing password: {}", err)
    }
}
fn handle_userdata(args: &Vec<String>, sqlite_conn: &Connection) {
    match args.len() {
        _i if 2 >= _i  => {
            println!("user empty");
            return;
        }
        _ => {}
    }
    let string = userdata(&args[2], sqlite_conn);
    match string {
        Ok(msg) => println!("{}", msg),
        Err(err) => println!("Error fetching user data: {}", err)
     }
}
fn handle_users_report(sqlite_conn: &Connection) {
   let string = users_report_json(sqlite_conn);
    match string {
       Ok(msg) => println!("{}", msg),
        Err(err) => println!("Error generating users report: {}", err)
    }
}
fn handle_expired_report(sqlite_conn: &Connection) {
    let string = expired_report_json(sqlite_conn);
    match string {
        Ok(msg) => println!("{}", msg),
        Err(err) => println!("Error generating expired report: {}", err)
    }
}
fn handle_online_report(sqlite_conn: &Connection) {
   let string = online_report_json(sqlite_conn);
    match string {
       Ok(msg) => println!("{}", msg),
        Err(err) => println!("Error generating online report: {}", err)
    }
}
fn print_help() {
    let mut text = " -- help data".to_owned();
    text = text + "\n   --create-user <user> <pass> <days> <limit>";
    text = text + "\n   --remove-user <user>";
    text = text + "\n   --generate-test <time in minutes>";
    text = text + "\n   --change-limit <user> <limit>";
    text = text + "\n   --change-validity <user> <validity in days>";
    text = text + "\n   --change-pass <user> <pass>";
    text = text + "\n   --users-report";
    text = text + "\n   --expired-report";
    text = text + "\n   --online-report";
    println!("{}", text)
}

fn user_dont_exists() {
    println!("esse não existe\n\n> Pressione qualquer tecla para voltar ao menu");
    let mut return_string = String::new();
    io::stdin().read_line(&mut return_string).expect("");
}

fn user_exists() {
    println!("esse usuario já existe\n\n> Pressione qualquer tecla para voltar ao menu");
    let mut return_string = String::new();
    io::stdin().read_line(&mut return_string).expect("");
}
                    let mut return_string = String::new();
                        io::stdin().read_line(&mut return_string).expect("");
                    }
                }
            }
            Err(_) => {
                Command::new("clear").status().unwrap();
                println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                let mut return_string = String::new();
                io::stdin().read_line(&mut return_string).expect("");
            }
        }


    }
}

fn utils_menu(sqlite_conn: &Connection) {
    loop {
        Command::new("clear").status().unwrap();
        
        println!("------------------------------------------------");
        println!("|                  {}                 |", text_to_bold("Ferramentas"));
        println!("------------------------------------------------");
        println!("| {:<45}|", "1 - Checkuser Multi-Apps");
        println!("| {:<45}|", "2 - Teste de Velocidade");
        println!("| {:<45}|", "3 - Monitorar recursos");
        println!("| {:<45}|", "4 - Gerenciar Journald");
        println!("| {:<45}|", "0 - Voltar ao menu");
        println!("------------------------------------------------");
        println!();
        let mut option = String::new();
        println!(" --> Selecione uma opção:");
        io::stdin().read_line(&mut option).unwrap();
        match option.trim().parse() {
            Ok(op) => {
                match op {
                    1 => {
                        checkuser_menu(&sqlite_conn);
                    }
                    2 => {
                        Command::new("clear").status().unwrap();
                        println!("teste em execução, essa ação pode demorar...");
                         match speedtest_data() {
                            Ok(speedtest) => {
                                let download_bits = speedtest.download.bytes as f64 * 8.0;
                                let upload_bits = speedtest.upload.bytes as f64 * 8.0;

                                let download_mb = download_bits / 1_000_000.0;
                                let upload_mb = upload_bits / 1_000_000.0;

                                let download_seconds = speedtest.download.elapsed as f64 / 1000.0;
                                let upload_seconds = speedtest.upload.elapsed as f64 / 1000.0;

                                let download_mbps = download_mb / download_seconds;
                                let upload_mbps = upload_mb / upload_seconds;

                                Command::new("clear").status().unwrap();

                                println!("------------------------------------------------");
                                println!("|              {}             |", text_to_bold("Teste de Velocidade"));
                                println!("------------------------------------------------");
                                println!("| Rede: {:<38} |", speedtest.interface.name);
                                println!("| Ip: {:<40} |", speedtest.interface.internal_ip);
                                println!("| Download: {:<34} |", format!("{:.2}mbps", download_mbps));
                                println!("| Download: {:<34} |", format!("{:.2}mbps", download_mbps));
                                println!("| Upload:   {:<34} |", format!("{:.2}mbps", upload_mbps));
                                println!("| Ping:     {:<32}   |", format!("{:.2}ms", speedtest.ping.latency));
                                println!("------------------------------------------------");

                                 println!("\n> pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");

                            }
                             Err(e) => {
                                 println!("Erro ao realizar speedtest: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                        }
                    }
                    3 => {
                        Command::new("clear").status().unwrap();
                        println!("{}", text_to_bold("> aviso: para sair do monitor, pressione F10"));
                        println!("> pressione qualquer tecla para continuar");
                        let mut return_string = String::new();
                        io::stdin().read_line(&mut return_string).expect("");
                        Command::new("htop").status().unwrap();
                    }
                    4 => {
                        journald_menu();
                    }
                    0 => {
                        break
                    }
                    _ => {
                        continue
                    }
                }
            }
            _ => {
                Command::new("clear").status().unwrap();
                println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                let mut return_string = String::new();
                io::stdin().read_line(&mut return_string).expect("");
            }
        }
    }
}
fn proxy_menu(sqlite_conn: &Connection) {
    loop {
        Command::new("clear").status().unwrap();
        
        println!("------------------------------------------------");
        println!("|                  {}                 |", text_to_bold("RUSTY PROXY"));
        println!("------------------------------------------------");
         match get_connections(&sqlite_conn) {
             Ok(conn) => {
                 let proxy_ports = conn.proxy.ports.unwrap_or_default();
                if proxy_ports.is_empty() {
                    println!("| Portas(s): {:<34}|", "nenhuma");
                } else {
                    let active_ports = proxy_ports.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(" ");
                   println!("| Portas(s): {:<34}|", active_ports);
                }
             }
             Err(e) => println!("Erro ao buscar portas: {}", e)
         }

        println!("------------------------------------------------");
        println!("| {:<45}|", "1 - Abrir Porta");
        println!("| {:<45}|", "2 - Fechar Porta");
        println!("| {:<45}|", "0 - Voltar ao menu");
        println!("------------------------------------------------");
        println!();
        let mut option = String::new();
        println!(" --> Selecione uma opção:");
        io::stdin().read_line(&mut option).unwrap();
        match option.trim().parse() {
            Ok(op) => {
                match op {
                    1 => {
                        let mut port = String::new();
                        loop {
                            println!("Digite a porta: ");
                            if !port.is_empty() {
                                port = String::new();
                            };
                            io::stdin().read_line(&mut port).unwrap();
                            port = port.trim().to_string();
                            match port.parse::<usize>() {
                                Ok(port) => {
                                    match is_port_avaliable(port) {
                                        Ok(true) => {
                                        }
                                        Ok(false) => {
                                            println!("essa porta já está em uso, digite outra:")
                                            continue
                                        }
                                        Err(_) => {
                                             println!("Erro ao verificar porta, tente novamente:")
                                             continue
                                        }
                                    }
                                    break
                                }
                                Err(..) => {
                                    println!("digite uma porta valida");
                                }
                            }

                        }
                        println!("Digite o status de conexão (não digite nada para o padrão): ");
                        let mut status = String::new();
                        io::stdin().read_line(&mut status).unwrap();
                        status = status.trim().to_string();

                         match enable_proxy_port(port, status){
                            Ok(_) => {
                                Command::new("clear").status().unwrap();
                                println!("\n> Porta ativada com sucesso, pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                             Err(e) => {
                                println!("Erro ao abrir porta: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                             }
                        }

                    }
                    2 => {
                        let mut port = String::new();
                        loop {
                            println!("Digite a porta: ");
                            if !port.is_empty() {
                                port = String::new();
                            };
                            io::stdin().read_line(&mut port).unwrap();
                            port = port.trim().to_string();
                            match port.parse::<usize>() {
                                Ok(port) => {
                                    match is_port_avaliable(port){
                                        Ok(false) => {},
                                        Ok(true) => {
                                             println!("essa porta não está em uso, digite outra:")
                                             continue
                                        }
                                        Err(_) => {
                                             println!("Erro ao verificar porta, tente novamente:")
                                             continue
                                         }
                                    }

                                    break
                                }
                                Err(..) => {
                                    println!("digite uma porta valida");
                                }
                            }

                        }

                        match disable_proxy_port(port) {
                             Ok(_) => {
                                 Command::new("clear").status().unwrap();
                                println!("\n> Porta desativada com sucesso, pressione qualquer tecla para voltar ao menu");
                                 let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                            Err(e) => {
                                 println!("Erro ao fechar porta: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                 let mut return_string = String::new();
                                 io::stdin().read_line(&mut return_string).expect("");
                             }
                        }


                    }
                    0 => {
                        break
                    }
                    _ => {
                        continue
                    }
                }
            }
            _ => {
                Command::new("clear").status().unwrap();
                println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                let mut return_string = String::new();
                io::stdin().read_line(&mut return_string).expect("");
            }
        }
    }
}
fn stunnel_menu(sqlite_conn: &Connection) {
    loop {
        Command::new("clear").status().unwrap();
        
        println!("------------------------------------------------");
        println!("|                    {}                   |", text_to_bold("STUNNEL"));
        println!("------------------------------------------------");
        match get_connections(&sqlite_conn) {
            Ok(conn) => {
                let stunnel_ports = conn.stunnel.ports.unwrap_or_default();
                 if stunnel_ports.is_empty() {
                      println!("| Portas(s): {:<34}|", "nenhuma");
                 } else {
                    let active_ports = stunnel_ports.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(" ");
                     println!("| Portas(s): {:<34}|", active_ports);
                 }

            }
             Err(e) => println!("Erro ao buscar portas: {}", e)
        }

        println!("| 1 - {:<40} |", "Abrir Porta");
        println!("| 2 - {:<40} |", "Abrir Porta Ipv6 (usuarios avançados)");
        println!("| 3 - {:<40} |", "Fechar Porta");
        println!("| 0 - {:<40} |", "Voltar ao menu");
        println!("------------------------------------------------");
        let mut option = String::new();
        println!("\n --> Selecione uma opção:");
        io::stdin().read_line(&mut option).unwrap();
        match option.trim().parse() {
            Ok(op) => {
                match op {
                     1 => {
                         let mut port = String::new();
                        loop {
                            println!("Digite a porta: ");
                           if !port.is_empty() {
                                 port = String::new();
                            };
                           io::stdin().read_line(&mut port).unwrap();
                            port = port.trim().to_string();
                           match port.parse::<usize>() {
                                 Ok(port) => {
                                     match is_port_avaliable(port){
                                         Ok(true) => {},
                                          Ok(false) => {
                                             println!("essa porta já está em uso, digite outra:")
                                            continue
                                         }
                                          Err(_) => {
                                             println!("Erro ao verificar porta, tente novamente:")
                                             continue
                                          }
                                    }
                                     break
                                  }
                                  Err(..) => {
                                     println!("digite uma porta valida");
                                  }
                            }
                        }
                        match enable_stunnel_port(port, false) {
                            Ok(_) => {
                                Command::new("clear").status().unwrap();
                                println!("\n> Porta ativada com sucesso, pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                            Err(e) => {
                                 println!("Erro ao abrir porta: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                             }
                        }
                    }
                    2 => {
                       let mut port = String::new();
                         loop {
                            println!("Digite a porta: ");
                            if !port.is_empty() {
                                 port = String::new();
                            };
                            io::stdin().read_line(&mut port).unwrap();
                             port = port.trim().to_string();
                             match port.parse::<usize>() {
                                Ok(port) => {
                                     match is_port_avaliable(port){
                                        Ok(true) => {},
                                         Ok(false) => {
                                             println!("essa porta já está em uso, digite outra:")
                                            continue
                                         }
                                          Err(_) => {
                                             println!("Erro ao verificar porta, tente novamente:")
                                             continue
                                          }
                                    }
                                   break
                                 }
                                 Err(..) => {
                                     println!("digite uma porta valida");
                                 }
                             }
                        }
                         match enable_stunnel_port(port, true) {
                             Ok(_) => {
                                Command::new("clear").status().unwrap();
                                println!("\n> Porta ativada com sucesso, pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                             Err(e) => {
                                 println!("Erro ao abrir porta: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                             }
                         }
                    }


                    3 => {
                        let mut port = String::new();
                        loop {
                            println!("Digite a porta: ");
                            if !port.is_empty() {
                                port = String::new();
                            };
                            io::stdin().read_line(&mut port).unwrap();
                            port = port.trim().to_string();
                             match port.parse::<usize>() {
                                Ok(port) => {
                                    match is_port_avaliable(port){
                                         Ok(false) => {},
                                          Ok(true) => {
                                             println!("essa porta não está em uso, digite outra:")
                                            continue
                                         }
                                          Err(_) => {
                                             println!("Erro ao verificar porta, tente novamente:")
                                             continue
                                          }
                                    }
                                     break
                                 }
                                Err(..) => {
                                     println!("digite uma porta valida");
                                 }
                           }
                        }

                        match disable_stunnel_port(port) {
                            Ok(_) => {
                                Command::new("clear").status().unwrap();
                                println!("\n> Porta desativada com sucesso, pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                             Err(e) => {
                                 println!("Erro ao fechar porta: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                 let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                             }
                        }

                    }
                    0 => {
                        break
                    }
                    _ => {
                        continue
                    }
                }
            }
            _ => {
                Command::new("clear").status().unwrap();
                println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                let mut return_string = String::new();
                io::stdin().read_line(&mut return_string).expect("");
            }
        }
    }
}
fn badvpn_menu(sqlite_conn: &Connection) {
    loop {
        Command::new("clear").status().unwrap();
        
        println!("------------------------------------------------");
        println!("|                    {}                    |", text_to_bold("BADVPN"));
        println!("------------------------------------------------");
         match get_connections(&sqlite_conn) {
            Ok(conn) => {
                 let badvpn_ports = conn.badvpn.ports.unwrap_or_default();
                 if badvpn_ports.is_empty() {
                     println!("| Portas(s): {:<34}|", "nenhuma");
                 } else {
                     let active_ports = badvpn_ports.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(" ");
                     println!("| Portas(s): {:<34}|", active_ports);
                 }

            }
            Err(e) => println!("Erro ao buscar portas: {}", e)
         }

        println!("| 1 - {:<40} |", "Abrir Porta");
        println!("| 2 - {:<40} |", "Fechar Porta");
        println!("| 0 - {:<40} |", "Voltar ao menu");
        println!("------------------------------------------------");
        let mut option = String::new();
        println!("\n --> Selecione uma opção:");
        io::stdin().read_line(&mut option).unwrap();
        match option.trim().parse() {
            Ok(op) => {
                match op {
                    1 => {
                       let mut port = String::new();
                         loop {
                            println!("Digite a porta: ");
                            if !port.is_empty() {
                                 port = String::new();
                            };
                           io::stdin().read_line(&mut port).unwrap();
                            port = port.trim().to_string();
                           match port.parse::<usize>() {
                                  Ok(port) => {
                                        match is_port_avaliable(port){
                                             Ok(true) => {},
                                             Ok(false) => {
                                                println!("essa porta já está em uso, digite outra:")
                                               continue
                                             }
                                             Err(_) => {
                                                println!("Erro ao verificar porta, tente novamente:")
                                                continue
                                             }
                                        }
                                         break
                                    }
                                   Err(..) => {
                                        println!("digite uma porta valida");
                                    }
                            }
                        }

                        match enable_badvpn_port(port) {
                            Ok(_) => {
                                Command::new("clear").status().unwrap();
                                println!("\n> Porta ativada com sucesso, pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                             Err(e) => {
                                 println!("Erro ao abrir porta: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                 let mut return_string = String::new();
                                 io::stdin().read_line(&mut return_string).expect("");
                             }
                        }
                     
                    }
                    2 => {
                       let mut port = String::new();
                         loop {
                            println!("Digite a porta: ");
                           if !port.is_empty() {
                                port = String::new();
                           };
                            io::stdin().read_line(&mut port).unwrap();
                           port = port.trim().to_string();
                            match port.parse::<usize>() {
                                  Ok(port) => {
                                        match is_port_avaliable(port){
                                            Ok(false) => {},
                                             Ok(true) => {
                                                println!("essa porta não está em uso, digite outra:")
                                               continue
                                            }
                                            Err(_) => {
                                                println!("Erro ao verificar porta, tente novamente:")
                                               continue
                                            }
                                        }
                                        break
                                    }
                                   Err(..) => {
                                        println!("digite uma porta valida");
                                    }
                            }
                        }

                       match disable_badvpn_port(port) {
                            Ok(_) => {
                                 Command::new("clear").status().unwrap();
                                 println!("\n> Porta desativada com sucesso, pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                            Err(e) => {
                                 println!("Erro ao fechar porta: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                 let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                        }
                    }
                    0 => {
                        break
                    }
                    _ => {
                        continue
                    }
                }
            }
            _ => {
                Command::new("clear").status().unwrap();
                println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                let mut return_string = String::new();
                io::stdin().read_line(&mut return_string).expect("");
            }
        }
    }
}
fn checkuser_menu(sqlite_conn: &Connection) {
    loop {
         Command::new("clear").status().unwrap();
        
        println!("------------------------------------------------");
        println!("|                   {}                  |", text_to_bold("CHECKUSER"));
        println!("------------------------------------------------");
        match get_connections(&sqlite_conn) {
            Ok(conn) => {
                let checkuser_ports = conn.checkuser.ports.unwrap_or_default();
                 if checkuser_ports.is_empty() {
                    println!("| Portas(s): {:<34}|", "nenhuma");
                } else {
                     let active_ports = checkuser_ports.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(" ");
                     println!("| Portas(s): {:<34}|", active_ports);
                }
            }
             Err(e) => println!("Erro ao buscar portas: {}", e)
        }
         println!("| 1 - {:<40} |", "Abrir Porta");
        println!("| 2 - {:<40} |", "Fechar Porta");
        println!("| 0 - {:<40} |", "Voltar ao menu");
        println!("------------------------------------------------");
         let mut option = String::new();
         println!("\n --> Selecione uma opção:");
         io::stdin().read_line(&mut option).unwrap();
        match option.trim().parse() {
           Ok(op) => {
                match op {
                     1 => {
                          let mut port = String::new();
                         loop {
                             println!("Digite a porta: ");
                            if !port.is_empty() {
                                  port = String::new();
                           };
                            io::stdin().read_line(&mut port).unwrap();
                           port = port.trim().to_string();
                           match port.parse::<usize>() {
                                 Ok(port) => {
                                     match is_port_avaliable(port){
                                         Ok(true) => {},
                                          Ok(false) => {
                                             println!("essa porta já está em uso, digite outra:")
                                            continue
                                         }
                                          Err(_) => {
                                             println!("Erro ao verificar porta, tente novamente:")
                                             continue
                                          }
                                     }
                                     break
                                  }
                                  Err(..) => {
                                      println!("digite uma porta valida");
                                  }
                            }
                         }
                         match enable_checkuser_port(port) {
                            Ok(_) => {
                                Command::new("clear").status().unwrap();
                                println!("\n> Porta ativada com sucesso, pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                             Err(e) => {
                                  println!("Erro ao abrir porta: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                 let mut return_string = String::new();
                                 io::stdin().read_line(&mut return_string).expect("");
                             }
                        }


                     }
                     2 => {
                         let mut port = String::new();
                         loop {
                             println!("Digite a porta: ");
                             if !port.is_empty() {
                                  port = String::new();
                            };
                            io::stdin().read_line(&mut port).unwrap();
                            port = port.trim().to_string();
                             match port.parse::<usize>() {
                                Ok(port) => {
                                      match is_port_avaliable(port){
                                          Ok(false) => {},
                                            Ok(true) => {
                                                println!("essa porta não está em uso, digite outra:")
                                              continue
                                           }
                                            Err(_) => {
                                                println!("Erro ao verificar porta, tente novamente:")
                                                continue
                                            }
                                      }
                                     break
                                }
                                Err(..) => {
                                      println!("digite uma porta valida");
                                  }
                             }
                         }
                         match disable_checkuser_port(port){
                             Ok(_) => {
                                Command::new("clear").status().unwrap();
                                println!("\n> Porta desativada com sucesso, pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                            }
                            Err(e) => {
                                  println!("Erro ao fechar porta: {}\n\n> Pressione qualquer tecla para voltar ao menu", e);
                                 let mut return_string = String::new();
                                io::stdin().read_line(&mut return_string).expect("");
                             }
                         }
                     }
                    0 => {
                        break
                    }
                    _ => {
                        continue
                    }
                }
           }
            _ => {
               Command::new("clear").status().unwrap();
                println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                let mut return_string = String::new();
                io::stdin().read_line(&mut return_string).expect("");
           }
        }
    }
}
fn journald_menu() {
    loop {
        Command::new("clear").status().unwrap();
        
        println!("------------------------------------------------");
        println!("|               {}              |", text_to_bold("Gerenciar Journald"));
        println!("------------------------------------------------");
        match journald_status(){
            Ok(status) => {
                if status {
                    println!("| Status: {:<37}|", "ativado");
                    println!("------------------------------------------------");
                    println!("| 1 - {:<40} |", "Desativar");
                 } else {
                    println!("| Status: {:<37}|", "desativado");
                    println!("------------------------------------------------");
                    println!("| 1 - {:<40} |", "Ativar");
                 }
                println!("| 0 - {:<40} |", "Voltar ao menu");
                println!("------------------------------------------------");
                let mut option = String::new();
                println!("\n --> Selecione uma opção:");
                 io::stdin().read_line(&mut option).unwrap();
                match option.trim().parse() {
                    Ok(op) => {
                       match op {
                           1 => {
                               if status {
                                  match disable_journald(){
                                      Ok(_) => {}
                                      Err(e) => println!("Erro ao desativar journald: {}", e)
                                  }
                               } else {
                                   match enable_journald(){
                                         Ok(_) => {},
                                       Err(e) => println!("Erro ao ativar journald: {}", e)
                                   }
                               }
                                Command::new("clear").status().unwrap();
                                println!("\n> Sucesso, pressione qualquer tecla para voltar ao menu");
                                let mut return_string = String::new();
                                 io::stdin().read_line(&mut return_string).expect("");
                            }
                            0 => {
                                break
                            }
                           _ => {
                                continue
                            }
                       }
                   }
                   _ => {
                        Command::new("clear").status().unwrap();
                        println!("\n> Opção invalida, pressione qualquer tecla para voltar ao menu");
                       let mut return_string = String::new();
                        io::stdin().read_line(&mut return_string).expect("");
                   }
               }
            }
           Err(e) => println!("Erro ao buscar status journald: {}", e)
        }
    }
}
fn services_menu() {
    Command::new("clear").status().unwrap();

    println!("------------------------------------------------");
    println!("|                 {}                |", text_to_bold("Portas Ativas"));
    println!("------------------------------------------------");
    match get_services() {
        Ok(services) => {
             for service in services {
                println!("| - {:<43}|", format!("{}: {}", service.name, service.ports.iter().map(|p| p.to_string()).collect::<Vec<String>>().join(" ")));
            }
        }
         Err(e) => println!("Erro ao buscar portas ativas: {}", e)
    }
   
    println!("------------------------------------------------");
    println!();
    println!("> Pressione qualquer tecla para voltar ao menu");
    let mut return_string = String::new();
    io::stdin().read_line(&mut return_string).expect("");
}
