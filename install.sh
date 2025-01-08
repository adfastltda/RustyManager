#!/bin/bash
# RustyManager Installer

set -e

TOTAL_STEPS=15
CURRENT_STEP=0

show_progress() {
    PERCENT=$((CURRENT_STEP * 100 / TOTAL_STEPS))
    echo "Progresso: [${PERCENT}%] - $1"
}

error_exit() {
    echo -e "\nErro: $1"
    exit 1
}

increment_step() {
    CURRENT_STEP=$((CURRENT_STEP + 1))
}

check_command() {
    if ! command -v "$1" &> /dev/null; then
        return 1
    else
        return 0
    fi
}
install_system_packages() {
    show_progress "Atualizando repositorios..."
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -y > update.log 2>&1
    if [ "$?" -ne 0 ]; then
        error_exit "Falha ao atualizar os repositorios"
    fi
    increment_step

    show_progress "Verificando o sistema..."
    if ! check_command lsb_release; then
        apt-get install lsb-release -y > install_lsb.log 2>&1
        if [ "$?" -ne 0 ]; then
            error_exit "Falha ao instalar lsb-release"
        fi
    fi
    increment_step

     OS_NAME=$(lsb_release -is)
    VERSION=$(lsb_release -rs)

    case $OS_NAME in
        Ubuntu)
            case $VERSION in
                24.*|22.*|20.*|18.*)
                    show_progress "Sistema Ubuntu suportado, continuando..."
                    ;;
                *)
                    error_exit "Versão do Ubuntu não suportada. Use 18, 20, 22 ou 24."
                    ;;
            esac
            ;;
        Debian)
            case $VERSION in
                12*|11*|10*|9*)
                    show_progress "Sistema Debian suportado, continuando..."
                    ;;
                *)
                    error_exit "Versão do Debian não suportada. Use 9, 10, 11 ou 12."
                    ;;
            esac
            ;;
        *)
            error_exit "Sistema não suportado. Use Ubuntu ou Debian."
            ;;
    esac
    increment_step

    show_progress "Atualizando o sistema..."
    apt-get upgrade -y > upgrade.log 2>&1
    if [ "$?" -ne 0 ]; then
        error_exit "Falha ao atualizar o sistema"
    fi
    
    apt-get install gnupg curl build-essential git cmake sysstat net-tools sqlite3 libsqlite3-dev -y > install_packages.log 2>&1
        if [ "$?" -ne 0 ]; then
           error_exit "Falha ao instalar pacotes"
        fi
    increment_step
}
setup_directories() {
    show_progress "Criando diretorio /opt/rustymanager..."
     mkdir -p /opt/rustymanager
     increment_step
}
setup_database() {
    show_progress "Configurando o banco de dados..."
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
    "
     if [ "$?" -ne 0 ]; then
        error_exit "Falha ao configurar o banco de dados"
    fi
    increment_step
}

install_rust() {
    show_progress "Instalando Rust..."
     if ! check_command rustc; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y > install_rust.log 2>&1
        if [ "$?" -ne 0 ]; then
            error_exit "Falha ao instalar Rust"
        fi
        source "$HOME/.cargo/env"
    fi
    increment_step
}
install_rustymanager() {
    show_progress "Compilando RustyManager, isso pode levar bastante tempo dependendo da maquina..."
    mkdir -p /opt/rustymanager
    git clone --branch "$SCRIPT_VERSION" --recurse-submodules --single-branch https://github.com/adfastltda/RustyManager.git /root/RustyManager > clone_rustymanager.log 2>&1
    if [ "$?" -ne 0 ]; then
        error_exit "Falha ao clonar RustyManager"
    fi

    cd /root/RustyManager/
    cargo build --release --jobs $(nproc) > build_rustymanager.log 2>&1
    if [ "$?" -ne 0 ]; then
        error_exit "Falha ao compilar RustyManager"
    fi
    cp ./target/release/SshScript /opt/rustymanager/manager
    cp ./target/release/CheckUser /opt/rustymanager/checkuser
    cp ./target/release/RustyProxy /opt/rustymanager/rustyproxy
    cp ./target/release/ConnectionsManager /opt/rustymanager/connectionsmanager
    increment_step
}
compile_badvpn(){
    show_progress "Compilando BadVPN..."
    mkdir -p /root/RustyManager/BadVpn/badvpn/badvpn-build
    cd /root/RustyManager/BadVpn/badvpn/badvpn-build
    cmake .. -DBUILD_NOTHING_BY_DEFAULT=1 -DBUILD_UDPGW=1 > cmake_badvpn.log 2>&1
        if [ "$?" -ne 0 ]; then
            error_exit "Falha ao configurar cmake para BadVPN"
        fi
    make -j$(nproc) > make_badvpn.log 2>&1
     if [ "$?" -ne 0 ]; then
        error_exit "Falha ao compilar BadVPN"
    fi
    mv udpgw/badvpn-udpgw /opt/rustymanager/badvpn
    increment_step
}
setup_permissions(){
    show_progress "Configurando permissões..."
    chmod +x /opt/rustymanager/{manager,proxy,connectionsmanager,checkuser,badvpn}
    ln -s /opt/rustymanager/manager /usr/local/bin/menu
    increment_step
}
install_stunnel(){
    show_progress "Instalando STunnel..."
    apt-get install -y stunnel4 > install_stunnel.log 2>&1
     if [ "$?" -ne 0 ]; then
       error_exit "Falha ao instalar STunnel"
    fi
    curl -o /etc/stunnel/cert.pem https://raw.githubusercontent.com/adfastltda/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/cert.pem > download_cert.log 2>&1
     if [ "$?" -ne 0 ]; then
       error_exit "Falha ao baixar cert.pem"
    fi
    curl -o /etc/stunnel/key.pem https://raw.githubusercontent.com/adfastltda/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/key.pem > download_key.log 2>&1
     if [ "$?" -ne 0 ]; then
        error_exit "Falha ao baixar key.pem"
    fi
    curl -o /etc/stunnel/stunnel.conf https://raw.githubusercontent.com/adfastltda/RustyManager/refs/heads/$SCRIPT_VERSION/Utils/stunnel/conf > download_conf.log 2>&1
     if [ "$?" -ne 0 ]; then
        error_exit "Falha ao baixar config"
    fi

    sed -i 's/ENABLED=0/ENABLED=1/g' /etc/default/stunnel4 || error_exit "Falha ao configurar STunnel"
    systemctl is-active stunnel4 > /dev/null 2>&1
    if [ "$?" -eq 0 ]; then
        systemctl stop stunnel4 > stop_stunnel.log 2>&1
        if [ "$?" -ne 0 ]; then
            error_exit "Falha ao parar stunnel"
        fi
        systemctl disable stunnel4 > disable_stunnel.log 2>&1
        if [ "$?" -ne 0 ]; then
             error_exit "Falha ao desabilitar stunnel"
        fi
    fi
    increment_step
}
install_speedtest() {
    show_progress "Instalando Speedtest..."
    curl -s https://packagecloud.io/install/repositories/ookla/speedtest-cli/script.deb.sh > speedtest_script.sh
    bash speedtest_script.sh > install_speedtest_script.log 2>&1
    if [ "$?" -ne 0 ]; then
        error_exit "Falha ao baixar e instalar o script do speedtest"
    fi
    apt-get install -y speedtest > install_speedtest.log 2>&1
      if [ "$?" -ne 0 ]; then
        error_exit "Falha ao instalar o speedtest"
    fi
    increment_step
}
install_htop(){
    show_progress "Instalando monitor de recursos..."
    apt-get install -y htop > install_htop.log 2>&1
     if [ "$?" -ne 0 ]; then
       error_exit "Falha ao instalar o speedtest"
    fi
    increment_step
}
cleanup_temp(){
    show_progress "Limpando diretórios temporários..."
    rm -rf /root/RustyManager/
    increment_step
}
# ---->>>> Main script
if [ "$EUID" -ne 0 ]; then
    error_exit "EXECUTE COMO ROOT"
else
    clear
    install_system_packages
    setup_directories
    setup_database
    install_rust
    install_rustymanager
    compile_badvpn
    setup_permissions
    install_stunnel
    install_speedtest
    install_htop
    cleanup_temp

    echo "Instalação concluída com sucesso. digite 'menu' para acessar o menu."
fi
