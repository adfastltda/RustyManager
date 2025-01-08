#!/bin/bash
# RustyManager Installer - Enhanced Version

set -e # Sai imediatamente em caso de erro
TOTAL_STEPS=16
CURRENT_STEP=0

show_progress() {
    local percent=$((CURRENT_STEP * 100 / TOTAL_STEPS))
    echo -e "\e[36m[${percent}%]\e[0m - $1"
}

error_exit() {
    echo -e "\n\e[31mErro:\e[0m $1" >&2
    exit 1
}

increment_step() {
    CURRENT_STEP=$((CURRENT_STEP + 1))
}

# Checa se está sendo executado como root
if [ "$EUID" -ne 0 ]; then
    error_exit "Este script deve ser executado como root."
fi

clear
show_progress "Inicializando a instalação..."
export DEBIAN_FRONTEND=noninteractive
increment_step

# Atualiza o sistema
show_progress "Atualizando repositórios e pacotes do sistema..."
apt-get update -y > /dev/null 2>&1 || error_exit "Falha ao atualizar os repositórios."
apt-get upgrade -y > /dev/null 2>&1 || error_exit "Falha ao atualizar o sistema."
increment_step

# Instala dependências
show_progress "Instalando dependências..."
apt-get install -y gnupg curl build-essential git cmake sysstat net-tools sqlite3 libsqlite3-dev wget htop > /dev/null 2>&1 || error_exit "Falha ao instalar pacotes essenciais."
increment_step

# Detecta sistema operacional e versão
show_progress "Detectando sistema operacional..."
OS_NAME=$(lsb_release -is)
VERSION=$(lsb_release -rs)
case $OS_NAME in
    Ubuntu)
        case $VERSION in
            24.*|22.*|20.*|18.*)
                echo "Ubuntu version supported."
                ;;
            *)
                 error_exit "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24."
                 ;;
        esac
        ;;
    Debian)
        case $VERSION in
            12*|11*|10*|9*)
                 echo "Debian version supported."
                 ;;
            *)
                error_exit "Versão do Debian não suportada. Use 9, 10, 11 ou 12."
                ;;
        esac
        ;;
    *)
        error_exit "Sistema operacional não suportado. Use Ubuntu ou Debian."
        ;;
esac
increment_step

# Cria diretório
show_progress "Criando diretório de instalação /opt/rustymanager..."
mkdir -p /opt/rustymanager || error_exit "Falha ao criar diretório de instalação."
increment_step

# Configura banco de dados SQLite
show_progress "Configurando banco de dados..."
sqlite3 /opt/rustymanager/db "
    CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        login_type TEXT NOT NULL,
        login_user TEXT NOT NULL,
        login_pass TEXT NOT NULL,
        login_limit TEXT NOT NULL,
        login_expiry TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS connections (
        id INTEGER PRIMARY KEY,
        proxy_ports TEXT,
        stunnel_ports TEXT,
        badvpn_ports TEXT,
        checkuser_ports TEXT
    );
" || error_exit "Falha ao criar tabelas no banco de dados."
increment_step

# Instala Rust
show_progress "Instalando Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y > /dev/null 2>&1 || error_exit "Falha ao instalar Rust."
    . "$HOME/.cargo/env"
fi
increment_step

# Clona e compila o projeto Rust
show_progress "Clonando e compilando o RustyManager..."
SCRIPT_VERSION="main" # Define a branch
git clone --branch "$SCRIPT_VERSION" --recurse-submodules --single-branch https://github.com/adfastltda/RustyManager.git /opt/rustymanager/source  || error_exit "Falha ao clonar o repositório."
cd /opt/rustymanager/source
cargo build --release --jobs $(nproc)  || error_exit "Falha ao compilar o projeto."
increment_step

# Instala os binarios rust
show_progress "Instalando binários Rust..."
mv ./target/release/SshScript /opt/rustymanager/manager
mv ./target/release/CheckUser /opt/rustymanager/checkuser
mv ./target/release/RustyProxy /opt/rustymanager/rustyproxy
mv ./target/release/ConnectionsManager /opt/rustymanager/connectionsmanager
increment_step

# Compila BadVPN
show_progress "Compilando BadVPN..."
mkdir -p /opt/rustymanager/source/BadVpn/badvpn/badvpn-build
cd /opt/rustymanager/source/BadVpn/badvpn/badvpn-build
cmake .. -DBUILD_NOTHING_BY_DEFAULT=1 -DBUILD_UDPGW=1 || error_exit "Falha ao configurar o cmake para BadVPN."
make || error_exit "Falha ao compilar BadVPN."
mv udpgw/badvpn-udpgw /opt/rustymanager/badvpn
increment_step

# Configura permissões e symlink
show_progress "Configurando permissões e links..."
chmod +x /opt/rustymanager/{manager,checkuser,rustyproxy,connectionsmanager,badvpn}
ln -sf /opt/rustymanager/manager /usr/local/bin/menu || error_exit "Falha ao criar o symlink para o menu."
increment_step

# Instala e configura Stunnel
show_progress "Instalando e configurando STunnel..."
apt-get install -y stunnel4 || error_exit "Falha ao instalar STunnel."
wget -O /etc/stunnel/cert.pem "https://raw.githubusercontent.com/adfastltda/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/cert.pem" || error_exit "Falha ao baixar o cert.pem."
wget -O /etc/stunnel/key.pem "https://raw.githubusercontent.com/adfastltda/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/key.pem"  || error_exit "Falha ao baixar o key.pem."
wget -O /etc/stunnel/stunnel.conf "https://raw.githubusercontent.com/adfastltda/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/conf"  || error_exit "Falha ao baixar a configuração do stunnel."
sed -i 's/ENABLED=0/ENABLED=1/g' /etc/default/stunnel4 || error_exit "Falha ao habilitar STunnel."
systemctl stop stunnel4 > /dev/null 2>&1
systemctl disable stunnel4 > /dev/null 2>&1
increment_step

# Instala speedtest
show_progress "Instalando Speedtest CLI..."
curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh | bash > /dev/null 2>&1 || error_exit "Falha ao adicionar repositório do Speedtest."
apt-get install -y speedtest > /dev/null 2>&1 || error_exit "Falha ao instalar Speedtest."
increment_step

# Limpeza
show_progress "Limpando diretórios temporários..."
rm -rf /opt/rustymanager/source
increment_step

echo -e "\n\e[32mInstalação concluída com sucesso!\e[0m"
echo "Digite 'menu' para acessar o menu do RustyManager."
