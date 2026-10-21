# Ruder Cam Treiber

## Pi Setup
- Activate i2c in `raspi-config` 
- Activate Serial Port in `raspi-config` (disable login shell over serial)
- Disable wifi powersaving


### Disable Swap 
```bash 
sudo nano /etc/rpi/swap.conf
```
```
[Main]
Mechanism=none
```

### Media Mtx
Upload mediamtx.yml to mediamtx repository of the Raspberry Pi.

```bash
sudo nano /etc/systemd/system/mediamtx.service
```

Insert the following content:
```
[Unit]
Description=MediaMTX Service
After=network.target

[Service]
Type=simple
WorkingDirectory=/home/pi/mediamtx
ExecStart=/home/pi/mediamtx/mediamtx
Restart=always
RestartSec=5
User=pi

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl enable mediamtx.service
sudo systemctl start mediamtx.service
```

Stream accessible at: `https://<your-pi-ip>:8889/stream/`







## Optimize boot (after first treiber run)
```bash
sudo systemctl disable --now bluetooth.service
sudo systemctl disable --now NetworkManager-wait-online.service

sudo touch /etc/cloud/cloud-init.disabled
sudo systemctl disable --now cloud-init-local.service
sudo systemctl disable --now cloud-config.service
sudo systemctl disable --now cloud-final.service

sudo nmcli connection modify "netplan-wlan0-Internetz 2.4 GHz" connection.autoconnect no
sudo nmcli connection modify "netplan-eth0" connection.autoconnect no

sudo nmcli connection modify treiber_hotspot connection.autoconnect yes
sudo nmcli connection modify treiber_hotspot connection.autoconnect-priority 100
```