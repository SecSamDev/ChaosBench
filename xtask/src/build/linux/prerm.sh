#!/bin/sh

first_arg="$1"

echo "Executing prerm script"

if [ "$first_arg" = "remove" ]; then
    echo "Remove chaosbench agent"
    systemctl stop chaosbench
    rm -rf /var/lib/chaosbench
fi

exit 0