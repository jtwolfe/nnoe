#!/bin/bash
set -euo pipefail

# Nebula Certificate Expiration Check Script
# Checks all certificates and reports expiration status

CERT_DIR="${1:-/etc/nebula/certs}"
CA_DIR="${2:-/etc/nebula/ca}"
WARN_DAYS="${3:-30}"  # Warn if expiring within 30 days
CRIT_DAYS="${4:-7}"    # Critical if expiring within 7 days

echo "Checking Nebula certificate expiration..."
echo "Certificate directory: $CERT_DIR"
echo "Warning threshold: $WARN_DAYS days"
echo "Critical threshold: $CRIT_DAYS days"
echo ""

if [ ! -d "$CERT_DIR" ]; then
    echo "Error: Certificate directory not found: $CERT_DIR"
    exit 1
fi

# Check CA certificate
if [ -f "$CA_DIR/ca.crt" ]; then
    echo "CA Certificate:"
    CA_INFO=$(nebula-cert print -path "$CA_DIR/ca.crt" 2>&1 || true)
    if echo "$CA_INFO" | grep -q "notAfter"; then
        CA_EXPIRY=$(echo "$CA_INFO" | grep "notAfter" | awk '{print $2" "$3}')
        echo "  Expiry: $CA_EXPIRY"
        
        if command -v date &> /dev/null; then
            CA_EXPIRY_EPOCH=$(date -d "$CA_EXPIRY" +%s 2>/dev/null || echo "0")
            CURRENT_EPOCH=$(date +%s)
            CA_DAYS=$(( (CA_EXPIRY_EPOCH - CURRENT_EPOCH) / 86400 ))
            
            if [ "$CA_DAYS" -lt 0 ]; then
                echo "  Status: EXPIRED"
            elif [ "$CA_DAYS" -lt "$CRIT_DAYS" ]; then
                echo "  Status: CRITICAL (expires in $CA_DAYS days)"
            elif [ "$CA_DAYS" -lt "$WARN_DAYS" ]; then
                echo "  Status: WARNING (expires in $CA_DAYS days)"
            else
                echo "  Status: OK (expires in $CA_DAYS days)"
            fi
        fi
    fi
    echo ""
fi

# Check node certificates
CERT_COUNT=0
EXPIRED_COUNT=0
WARNING_COUNT=0
CRITICAL_COUNT=0

for CERT_FILE in "$CERT_DIR"/*.crt; do
    if [ -f "$CERT_FILE" ]; then
        CERT_COUNT=$((CERT_COUNT + 1))
        CERT_NAME=$(basename "$CERT_FILE" .crt)
        
        CERT_INFO=$(nebula-cert print -path "$CERT_FILE" 2>&1 || true)
        
        if echo "$CERT_INFO" | grep -q "notAfter"; then
            EXPIRY_DATE=$(echo "$CERT_INFO" | grep "notAfter" | awk '{print $2" "$3}')
            
            if command -v date &> /dev/null; then
                EXPIRY_EPOCH=$(date -d "$EXPIRY_DATE" +%s 2>/dev/null || echo "0")
                CURRENT_EPOCH=$(date +%s)
                DAYS_UNTIL_EXPIRY=$(( (EXPIRY_EPOCH - CURRENT_EPOCH) / 86400 ))
                
                STATUS="OK"
                if [ "$DAYS_UNTIL_EXPIRY" -lt 0 ]; then
                    STATUS="EXPIRED"
                    EXPIRED_COUNT=$((EXPIRED_COUNT + 1))
                elif [ "$DAYS_UNTIL_EXPIRY" -lt "$CRIT_DAYS" ]; then
                    STATUS="CRITICAL"
                    CRITICAL_COUNT=$((CRITICAL_COUNT + 1))
                elif [ "$DAYS_UNTIL_EXPIRY" -lt "$WARN_DAYS" ]; then
                    STATUS="WARNING"
                    WARNING_COUNT=$((WARNING_COUNT + 1))
                fi
                
                printf "  %-30s Expires: %-20s (%3d days) [%s]\n" \
                    "$CERT_NAME" "$EXPIRY_DATE" "$DAYS_UNTIL_EXPIRY" "$STATUS"
            else
                printf "  %-30s Expires: %s\n" "$CERT_NAME" "$EXPIRY_DATE"
            fi
        else
            printf "  %-30s [Could not parse expiration]\n" "$CERT_NAME"
        fi
    fi
done

echo ""
echo "Summary:"
echo "  Total certificates: $CERT_COUNT"
echo "  OK: $((CERT_COUNT - EXPIRED_COUNT - WARNING_COUNT - CRITICAL_COUNT))"
echo "  Warning ($WARN_DAYS-$CRIT_DAYS days): $WARNING_COUNT"
echo "  Critical (< $CRIT_DAYS days): $CRITICAL_COUNT"
echo "  Expired: $EXPIRED_COUNT"

if [ "$EXPIRED_COUNT" -gt 0 ] || [ "$CRITICAL_COUNT" -gt 0 ]; then
    echo ""
    echo "ACTION REQUIRED: Certificates need rotation!"
    exit 1
elif [ "$WARNING_COUNT" -gt 0 ]; then
    echo ""
    echo "WARNING: Some certificates are nearing expiration."
    exit 0
else
    exit 0
fi

