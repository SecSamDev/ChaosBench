#!/bin/sh

service_exists() {
    local n=$1
    if [ $(systemctl is-active --quiet chaosbench.service) ]; then
        return 0
    else
        return 1
    fi
}
if service_exists chaosbench; then
    systemctl daemon-reload
    systemctl stop chaosbench
fi

mkdir -p /var/log/chaosbench
mkdir -p /etc/chaosbench
mkdir -p /var/lib/chaosbench/

chown root:root -R /var/log/chaosbench/
chmod 600 -R /var/log/chaosbench/
chown root:root -R /etc/chaosbench/
chmod 600 -R /etc/chaosbench/
chown root:root -R /var/lib/chaosbench/
chmod 700 -R /var/lib/chaosbench/

exit 0