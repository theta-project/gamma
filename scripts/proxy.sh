#!/usr/bin/env sh
mitmdump --set tls_version_client_min=TLS1 \
         --set keep_host_header=true \
         --listen-port 443 \
         --mode reverse:http://localhost:8080
