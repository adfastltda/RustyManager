#!/usr/bin/python
# -*- coding: utf-8 -*-
import os
import sys
import time
import glob

def executar_comando(comando):
    """
    Executa um comando do sistema e retorna a saída.
    """
    resultado = os.popen(comando).read().strip()
    return resultado

def processar_arquivo(arquivo_entrada, remover):
    """
    Processa um arquivo específico, um usuário por vez.
    """
    try:
        # Lê o arquivo de entrada
        with open(arquivo_entrada, 'r') as file:
            linhas = file.readlines()

        for i, linha in enumerate(linhas, start=1):
            # Remove espaços extras e divide a linha em partes
            colunas = linha.strip().split()

            # Verifica se o número de colunas é válido
            if len(colunas) != 4 and len(colunas) != 5:
                print(f"[Linha {i}] Erro: Linha com número incorreto de informações: '{linha.strip()}'")
                continue

            if len(colunas) == 4:
                usuario, senha, dias, limite = colunas
            elif len(colunas) == 5:
                usuario, senha, dias, limite, _ = colunas

            # Verifica se algum dado está ausente (campo vazio ou inválido)
            if not usuario or not senha or not dias or not limite:
                print(f"[Linha {i}] Erro: Dados ausentes ou inválidos para o usuário '{usuario}'")
                continue

            # Verifica se o campo 'dias' é negativo
            if int(dias) < 0:
                continue  # Não exibe nada para usuários com dias negativos

            # Adiciona 1 dia ao valor de 'dias'
            dias = str(int(dias) + 1)

            # Se o script foi executado com o parâmetro --remover, remove o usuário
            if remover:
                comando_remover = f"/opt/rustymanager/manager --remove-user {usuario}"
                print(f"[Linha {i}] Executando: {comando_remover}")
                executar_comando(comando_remover)
                time.sleep(1)  # Pausa de 1 segundo entre os comandos

            # Executa o comando de criação de usuário com o novo valor de 'dias'
            comando_criar = f"/opt/rustymanager/manager --create-user {usuario} {senha} {dias} {limite}"
            print(f"[Linha {i}] Executando: {comando_criar}")
            resultado = executar_comando(comando_criar)

            time.sleep(1)  # Pausa de 1 segundo entre os comandos

        # Remover o arquivo após processá-lo
        os.remove(arquivo_entrada)

    except FileNotFoundError:
        print(f"Erro: Arquivo '{arquivo_entrada}' não encontrado.")
    except ValueError as ve:
        print(f"Erro ao converter valores numéricos: {ve}")
    except Exception as e:
        print(f"Erro inesperado: {e}")

def main():
    # Verifica se o script foi executado com o parâmetro --remover
    remover = '--remover' in sys.argv

    # Busca por arquivos .txt no diretório /root/
    arquivos_txt = glob.glob('/root/*.txt')

    if not arquivos_txt:
        print("Nenhum arquivo .txt encontrado em /root/.")
        sys.exit(1)

    # Processa o primeiro arquivo encontrado
    arquivo_entrada = arquivos_txt[0]
    processar_arquivo(arquivo_entrada, remover)

if __name__ == "__main__":
    main()
