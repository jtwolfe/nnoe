#!/bin/bash
set -euo pipefail

# etcd Member Management Script
# Add/remove members from etcd cluster

ETCD_ENDPOINT="${1:-http://127.0.0.1:2379}"
ACTION="${2:-list}"
MEMBER_NAME="${3:-}"
MEMBER_IP="${4:-}"

# Validate etcdctl availability
if ! command -v etcdctl &> /dev/null; then
    echo "Error: etcdctl not found. Please install etcd."
    exit 1
fi

# Validate endpoint connectivity
if ! etcdctl --endpoints="$ETCD_ENDPOINT" endpoint health &>/dev/null; then
    echo "Error: Cannot connect to etcd endpoint: $ETCD_ENDPOINT"
    echo "Please verify etcd is running and the endpoint is correct."
    exit 1
fi

case "$ACTION" in
    list)
        echo "Listing etcd cluster members:"
        echo ""
        etcdctl --endpoints="$ETCD_ENDPOINT" member list -w table
        echo ""
        echo "Cluster health:"
        etcdctl --endpoints="$ETCD_ENDPOINT" endpoint health
        ;;
    add)
        if [ -z "$MEMBER_NAME" ] || [ -z "$MEMBER_IP" ]; then
            echo "Usage: $0 <endpoint> add <member-name> <member-ip>"
            exit 1
        fi
        
        # Check if member already exists
        EXISTING=$(etcdctl --endpoints="$ETCD_ENDPOINT" member list | grep -c "$MEMBER_NAME" || true)
        if [ "$EXISTING" -gt 0 ]; then
            echo "Warning: Member '$MEMBER_NAME' may already exist in the cluster"
            echo "Current members:"
            etcdctl --endpoints="$ETCD_ENDPOINT" member list
            read -p "Continue anyway? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                exit 1
            fi
        fi
        
        echo "Adding member: $MEMBER_NAME ($MEMBER_IP)"
        echo ""
        
        # Add member and capture output
        ADD_OUTPUT=$(etcdctl --endpoints="$ETCD_ENDPOINT" member add "$MEMBER_NAME" \
            --peer-urls="http://$MEMBER_IP:2380" 2>&1)
        
        if [ $? -eq 0 ]; then
            echo "$ADD_OUTPUT"
            echo ""
            echo "Member added successfully!"
            echo "Next steps:"
            echo "1. On the new node, run bootstrap script with the cluster configuration shown above"
            echo "2. Start etcd on the new node"
            echo "3. Verify member is healthy: $0 $ETCD_ENDPOINT list"
        else
            echo "Error adding member:"
            echo "$ADD_OUTPUT"
            exit 1
        fi
        ;;
    remove)
        if [ -z "$MEMBER_NAME" ]; then
            echo "Usage: $0 <endpoint> remove <member-id>"
            echo ""
            echo "To find member ID, first list members:"
            echo "  $0 $ETCD_ENDPOINT list"
            exit 1
        fi
        
        # Verify member exists
        MEMBER_EXISTS=$(etcdctl --endpoints="$ETCD_ENDPOINT" member list | grep -c "$MEMBER_NAME" || true)
        if [ "$MEMBER_EXISTS" -eq 0 ]; then
            echo "Error: Member '$MEMBER_NAME' not found in cluster"
            echo ""
            echo "Current members:"
            etcdctl --endpoints="$ETCD_ENDPOINT" member list
            exit 1
        fi
        
        # Warning for quorum
        MEMBER_COUNT=$(etcdctl --endpoints="$ETCD_ENDPOINT" member list | wc -l)
        if [ "$MEMBER_COUNT" -le 3 ]; then
            echo "Warning: Removing member will leave cluster with $((MEMBER_COUNT - 1)) members"
            echo "etcd requires majority quorum. For 3 nodes, removing 1 will leave 2 (quorum)."
            read -p "Continue? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                exit 1
            fi
        fi
        
        echo "Removing member: $MEMBER_NAME"
        if etcdctl --endpoints="$ETCD_ENDPOINT" member remove "$MEMBER_NAME"; then
            echo "Member removed successfully"
            echo ""
            echo "Updated member list:"
            etcdctl --endpoints="$ETCD_ENDPOINT" member list
        else
            echo "Error removing member"
            exit 1
        fi
        ;;
    *)
        echo "Usage: $0 <endpoint> {list|add|remove}"
        echo ""
        echo "Examples:"
        echo "  $0 http://127.0.0.1:2379 list"
        echo "  $0 http://127.0.0.1:2379 add etcd-2 192.168.1.11"
        echo "  $0 http://127.0.0.1:2379 remove <member-id>"
        exit 1
        ;;
esac

