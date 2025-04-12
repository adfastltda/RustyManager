import socket
import os

def conectar_http(host, porta):
    sock = None  # Inicializa a variável sock

    try:
        # Estabelece a conexão TCP diretamente na porta 80
        sock = socket.create_connection((host, porta))
        print("Conectado a {} na porta {}".format(host, porta))

        # Envia uma solicitação HTTP GET simples
        requisicao = f"GET / HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
        sock.sendall(requisicao.encode('utf-8'))

        # Recebe a resposta
        resposta = sock.recv(4096).decode('utf-8')
        print("Resposta do servidor:")
        print(resposta)

    except Exception as e:
        erro = str(e)
        print("Erro ao conectar: {}".format(erro))

        # Reinicia o serviço stunnel4 se o erro for "Connection refused"
        if "Connection refused" in erro:
            print("Reiniciando o serviço websocket...")
            os.system("/usr/sbin/service websocket restart")

    finally:
        if sock:
            sock.close()  # Fecha o socket principal se estiver aberto

if __name__ == "__main__":
    host = "premium.adfast.com.br"
    porta = 80
    conectar_http(host, porta)
