# Work with growbe-mainboard on rasberry pi

The growbe-mainboard code is deploy on rasberry pi using the image that we build here : https://github.com/growbe2/pi-gen

# Connect by ssh

­```bash
# Connecting from the same local network (check the local connection information from growbe cloud)
ssh pi@[ip]
# Connection from the cloud
ssh root@api.dev.growbe.ca
ssh -p [proxy_port] pi@localhost
# Get the list of ssh process port
netstat -tulpn | grep ssh

# All the file for the process are store in /opt/growbe
ls -la /opt/growbe
drwxr-xr-x 4 root root     4096 27 déc 10:18 .
drwxr-xr-x 4 root root     4096 26 nov 00:52 ..
drwxr-xr-x 2 root root     4096  9 déc 00:08 autossh
-rwxr-xr-x 1 root root      280 26 nov 11:48 configure.sh
-rw-r--r-- 1 pi   pi      20480 27 déc 10:18 database.sqlite
-rw-r--r-- 1 pi   pi        305 23 déc 12:03 dev.json
-rwxr-xr-x 1 root root     1020 10 déc 14:39 download.sh
drwxr-xr-x 2 root root     4096 29 nov 13:03 fluent
-rwxr-xr-x 1 pi   pi   12881848 22 déc 02:07 growbe-mainboard
-rwxr-xr-x 1 root root      192 26 nov 11:47 init.sh

# Get the status of the process with the latest log
systemctl status growbe-mainboard@dev.service

# Get live log
journalctl -u growbe-mainboard@dev.service -f

# Manually update the process
cd /opt/growbe
./download.sh latest growbe-mainboard-arm-linux
./download.sh v0.1.2 growbe-mainboard-arm-linux

# Restart the process
sudo systemctl restart growbe-mainboard@dev.service
```