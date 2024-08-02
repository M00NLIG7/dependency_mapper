#!/bin/sh

 curl -X POST http://localhost:8080/api/node \
        -H "Content-Type: application/json" \
        -d '{
        "srcIP": "192.168.1.9",
        "os": "Windows"
    }'

curl -X POST http://localhost:8080/api/node \
        -H "Content-Type: application/json" \
        -d '{
        "srcIP": "192.168.1.11",
        "os": "Windows"
    }'

curl -X POST http://localhost:8080/addnode \
             -H "Content-Type: application/json" \
             -d '{
               "NodeType": "service",
               "Module": "core",
               "LocalPort": 4502,
               "LocalIp": "192.168.10.9",
               "RemotePort": 3306,
               "RemoteIp": "192.168.10.11",
               "Description": "Core yeah yeah yeah"
             }'
