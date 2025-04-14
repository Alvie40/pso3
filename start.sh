#!/bin/bash

# Verifica se ./pg_data existe e é um diretório
if [ ! -d "./pg_data" ]; then
    echo "Criando o diretório ./pg_data..."
    mkdir ./pg_data
    chmod 755 ./pg_data
else
    echo "Diretório ./pg_data já existe, usando o existente."
fi

# Inicia o Docker Compose
docker-compose up -d