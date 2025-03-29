use clap::Parser;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{self, AsyncWriteExt}; // io necessário para copy_bidirectional
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Porta na qual o servidor proxy escutará
    #[arg(long, default_value_t = 80)]
    port: u16,

    /// Mensagem de status a ser enviada no handshake inicial
    #[arg(long, default_value = "@RustyManager")]
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let listener = TcpListener::bind(format!("[::]:{}", args.port)).await?;
    println!("Iniciando serviço na porta: {}", args.port);

    // Compartilha a string de status com segurança entre as tasks
    let shared_status = Arc::new(args.status);

    start_proxy(listener, shared_status).await; // Renomeado de start_http para clareza
    Ok(())
}

async fn start_proxy(listener: TcpListener, status: Arc<String>) {
    loop {
        match listener.accept().await {
            Ok((client_stream, addr)) => {
                println!("Cliente conectado: {}", addr);
                // Clona o Arc para a nova task
                let status_clone = status.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(client_stream, status_clone).await {
                        eprintln!("Erro ao processar cliente {}: {}", addr, e);
                    } else {
                        println!("Cliente desconectado: {}", addr);
                    }
                });
            }
            Err(e) => {
                eprintln!("Erro ao aceitar conexão: {}", e);
            }
        }
    }
}

async fn handle_client(
    mut client_stream: TcpStream,
    status: Arc<String>,
) -> Result<(), Box<dyn Error>> {
    // 1. Enviar resposta inicial (simplificada)
    //    O cliente precisa esperar por isso antes de enviar dados, ou ignorar.
    client_stream
        .write_all(format!("HTTP/1.1 101 {}\r\n\r\n", *status).as_bytes())
        .await?;
    client_stream.flush().await?; // Garante que a resposta foi enviada

    // 2. Espiar (peek) os dados iniciais sem consumi-los
    let addr_proxy: &str; // Usar &str para endereços fixos

    // Define um timeout curto para o peek
    const PEEK_TIMEOUT_SECS: u64 = 2;

    let peek_result = timeout(
        Duration::from_secs(PEEK_TIMEOUT_SECS),
        peek_stream(&client_stream), // Passa referência imutável
    )
    .await;

    let data_str = match peek_result {
        Ok(Ok(data)) => {
            println!("Dados iniciais espiados: {:?}", data); // Log para debug
            data
        }
        Ok(Err(e)) => {
            // Erro durante a operação de peek
            eprintln!("Erro ao espiar dados do cliente: {}", e);
            String::new() // Assume padrão (SSH) em caso de erro no peek
        }
        Err(_) => {
            // Timeout ocorreu
            println!(
                "Timeout ({:?}s) ao esperar dados iniciais do cliente. Usando backend padrão.",
                PEEK_TIMEOUT_SECS
            );
            String::new() // Assume padrão (SSH) em caso de timeout
        }
    };

    // 3. Decidir o backend
    //    Se contiver "SSH" (case-sensitive) ou se o peek falhou/timeout/vazio, usa SSH.
    //    Caso contrário, assume OpenVPN.
    if data_str.contains("SSH") || data_str.is_empty() {
        addr_proxy = "127.0.0.1:22"; // Corrigido para localhost
    } else {
        addr_proxy = "127.0.0.1:1194"; // Corrigido para localhost
    }

    println!("Redirecionando para o backend: {}", addr_proxy);

    // 4. Conectar ao backend
    let server_stream_result = TcpStream::connect(addr_proxy).await;

    let mut server_stream = match server_stream_result {
        Ok(stream) => {
            println!("Conectado ao backend com sucesso: {}", addr_proxy);
            stream
        }
        Err(e) => {
            eprintln!("Falha ao conectar ao backend {}: {}", addr_proxy, e);
            // Opcional: Tentar enviar um erro para o cliente aqui?
            // Por simplicidade, apenas fechamos a conexão do cliente retornando Ok.
            return Ok(());
        }
    };

    // 5. Transferir dados bidirecionalmente
    //    Usa copy_bidirectional para eficiência e simplicidade.
    //    Não precisamos mais de Arc<Mutex<...>> ou da função transfer_data.
    match io::copy_bidirectional(&mut client_stream, &mut server_stream).await {
        Ok((to_server, to_client)) => {
            println!(
                "Transferência concluída. Cliente -> Servidor: {} bytes, Servidor -> Cliente: {} bytes",
                to_server, to_client
            );
        }
        Err(e) => {
            eprintln!("Erro durante a transferência de dados: {}", e);
        }
    }

    Ok(()) // Indica que o tratamento deste cliente terminou (com ou sem erro de transferência)
}

/// Espia os dados iniciais de um stream sem consumi-los.
async fn peek_stream(stream: &TcpStream) -> Result<String, io::Error> {
    let mut peek_buffer = vec![0; 1024]; // Buffer razoável para dados iniciais
    let bytes_peeked = stream.peek(&mut peek_buffer).await?;

    if bytes_peeked == 0 {
        // Cliente pode ter desconectado antes de enviar dados após o handshake inicial
        return Ok(String::new());
    }

    // Converte apenas os bytes espiados para String (com perda se não for UTF-8 válido)
    let data_str = String::from_utf8_lossy(&peek_buffer[..bytes_peeked]);
    Ok(data_str.to_string())
}
