#!/bin/bash
# Health check
curl -s http://localhost:8080/health | jq
