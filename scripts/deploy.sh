#!/bin/bash
set -euo pipefail

# Deployment script for NNOE

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DEPLOYMENT_TYPE="${1:-docker}"
DEPLOYMENT_MODE="${2:-dev}"

cd "$PROJECT_ROOT"

case "$DEPLOYMENT_TYPE" in
    docker)
        echo "Deploying with Docker Compose..."
        cd deployments/docker
        docker-compose -f "docker-compose.$DEPLOYMENT_MODE.yml" up -d
        ;;
    kubernetes)
        echo "Deploying to Kubernetes..."
        cd deployments/kubernetes
        kubectl apply -k .
        ;;
    ansible)
        echo "Deploying with Ansible..."
        cd deployments/ansible
        ansible-playbook -i inventory playbooks/deploy.yml
        ;;
    *)
        echo "Unknown deployment type: $DEPLOYMENT_TYPE"
        echo "Usage: $0 [docker|kubernetes|ansible] [dev|prod]"
        exit 1
        ;;
esac

echo "Deployment complete!"

