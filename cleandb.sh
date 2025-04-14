#!/bin/bash

# Pergunta ao usuário se deseja limpar tudo
echo "Isso removerá completamente o container pso3_postgres e o diretório ./pg_data."
echo "Todos os dados do banco serão perdidos. Deseja continuar? (y/n)"
read -r response

# Verifica a resposta
if [ "$response" = "y" ] || [ "$response" = "Y" ]; then
    echo "Parando e removendo o container..."
    docker-compose down

    echo "Removendo o diretório ./pg_data..."
    sudo rm -rf ./pg_data

    echo "Limpeza concluída!"
else
    echo "Operação cancelada."
fi