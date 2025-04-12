import socket
import ssl
import os

def conectar_https(host, porta):
    contexto = ssl._create_unverified_context()
    sock = None  # Inicializa a variável sock

    try:
        # Estabelece a conexão TCP
        sock = socket.create_connection((host, porta))
        conexao_ssl = contexto.wrap_socket(sock, server_hostname=host)
        print("Conectado a {} na porta {}".format(host, porta))

        # Envia uma solicitação HTTP GET simples
        requisicao = f"GET / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
        conexao_ssl.sendall(requisicao.encode('utf-8'))

        # Recebe a resposta
        resposta = conexao_ssl.recv(4096).decode('utf-8')
        print("Resposta do servidor:")
        print(resposta)

        conexao_ssl.close()  # Fecha a conexão SSL

    except Exception as e:
        erro = str(e)
        print("Erro ao conectar: {}".format(erro))

        # Reinicia o serviço stunnel4 se o erro for "Connection refused"
        if "Connection refused" in erro:
            print("Reiniciando o serviço stunnel4...")
            os.system("/usr/sbin/service stunnel4 restart")

    finally:
        if sock:
            sock.close()  # Fecha o socket principal se estiver aberto

if __name__ == "__main__":
    host = "premium.adfast.com.br"
    porta = 443
    conectar_https(host, porta)
