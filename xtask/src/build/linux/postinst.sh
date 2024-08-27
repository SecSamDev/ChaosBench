#!/bin/sh

systemctl daemon-reload
systemctl enable chaosbench
systemctl start chaosbench
exit 0