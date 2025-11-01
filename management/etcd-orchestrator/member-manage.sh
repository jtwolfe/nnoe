#!/bin/bash
set -euo pipefail

# etcd Member Management Script
# Add/remove members from etcd cluster

ETCD_ENDPOINT="${1:-http://127.0.0.1:2379}"
ACTION="${2:-list}"
MEMBER_NAME="${3:-}"
MEMBER_IP="${4:-}"

case "$ACTION" in
    list)
        echo "Listing etcd cluster members:"
        etcdctl --endpoints="$ETCD_ENDPOINT" member list
        ;;
    add)
        if [ -z "$MEMBER_NAME" ] || [ -z "$MEMBER_IP" ]; then
            echo "Usage: $0 <endpoint> add <member-name> <member-ip>"
            exit 1
        fi
        echo "Adding member: $MEMBER_NAME ($MEMBER_IP)"
        etcdctl --endpoints="$ETCD_ENDPOINT" member add "$MEMBER_NAME" \
            --peer-urls="http://$MEMBER_IP:2380"
        ;;
    remove)
        if [ -z "$MEMBER_NAME" ]; then
            echo "Usage: $0 <endpoint> remove <member-id>"
            exit 1
        fi
        echo "Removing member: $MEMBER_NAME"
        etcdctl --endpoints="$ETCD_ENDPOINT" member remove "$MEMBER_NAME"
        ;;
    *)
        echo "Usage: $0 <endpoint> {list|add|remove}"
        exit 1
        ;;
esac

