#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

echo "Building slit_controller in release mode..."
cargo build --release --package slit_controller

sudo mkdir -p /opt/slit_controller/bin
sudo mkdir -p /opt/slit_controller/config
sudo mkdir -p /var/log/slit_controller

echo "Checking if slit-controller service is running..."
if sudo systemctl is-active --quiet slit-controller.service; then
  echo "Stopping slit-controller service..."
  sudo systemctl stop slit-controller.service
  echo "Service stopped."
else
  echo "Service is not running, continuing with installation..."
fi

echo "Copying executable to /opt/slit_controller/bin..."
sudo cp "$PROJECT_DIR/target/release/slit_controller" /opt/slit_controller/bin/
sudo chmod +x /opt/slit_controller/bin/slit_controller

echo "Copying default config to /opt/slit_controller/config..."
sudo cp "$PROJECT_DIR/default_config.toml" /opt/slit_controller/config/

echo "Creating systemd service file..."
cat << EOF | sudo tee /etc/systemd/system/slit-controller.service > /dev/null
[Unit]
Description=Slit Controller Service
After=network.target

[Service]
Type=simple
User=root
Group=root
ExecStart=/opt/slit_controller/bin/slit_controller --config /opt/slit_controller/config/default_config.toml
ExecStartPost=/bin/bash -c 'sleep 1; chmod 666 /tmp/slit_controller.sock'
WorkingDirectory=/opt/slit_controller
Restart=always
RestartSec=5
StandardOutput=append:/var/log/slit_controller/stdout.log
StandardError=append:/var/log/slit_controller/stderr.log

# Create socket directory with proper permissions
PermissionsStartOnly=true
ExecStartPre=/bin/mkdir -p /tmp
ExecStartPre=/bin/rm -f /tmp/slit_controller.sock

[Install]
WantedBy=multi-user.target
EOF

echo "Reloading systemd..."
sudo systemctl daemon-reload

echo "Enabling slit-controller service to start on boot..."
sudo systemctl enable slit-controller.service
echo "Starting slit-controller service..."
sudo systemctl start slit-controller.service

echo ""
echo "Installation complete!"
echo ""
echo "To start the service now, run:"
echo "  sudo systemctl start slit-controller.service"
echo ""
echo "To check the service status, run:"
echo "  sudo systemctl status slit-controller.service"
echo ""
echo "To view logs, run:"
echo "  sudo journalctl -u slit-controller.service"
echo "  or check log files in /var/log/slit_controller/"
