#!/bin/bash
set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PACKAGE_DIR="$(dirname "$SCRIPT_DIR")"
PROJECT_DIR="$(dirname "$PACKAGE_DIR")"
cd "$PROJECT_DIR"

echo "Building cooled_slit_controller in release mode..."
cargo build --release --package cooled_slit_controller

sudo mkdir -p /opt/cooled_slit_controller/bin
sudo mkdir -p /opt/cooled_slit_controller/config
sudo mkdir -p /opt/cooled_slit_controller/scripts
sudo mkdir -p /var/log/cooled_slit_controller

echo "Checking if cooled-slit-controller service is running..."
if sudo systemctl is-active --quiet cooled-slit-controller.service; then
  echo "Stopping cooled-slit-controller service..."
  sudo systemctl stop cooled-slit-controller.service
  echo "Service stopped."
else
  echo "Service is not running, continuing with installation..."
fi

echo "Copying executable to /opt/cooled_slit_controller/bin..."
sudo cp "$PROJECT_DIR/target/release/cooled_slit_controller" /opt/cooled_slit_controller/bin/
sudo chmod +x /opt/cooled_slit_controller/bin/cooled_slit_controller

echo "Copying safe restart script..."
sudo cp "$SCRIPT_DIR/safe_restart.sh" /opt/cooled_slit_controller/scripts/
sudo chmod +x /opt/cooled_slit_controller/scripts/safe_restart.sh

# Check if config file already exists
CONFIG_PATH="/opt/cooled_slit_controller/config/default_config.toml"
if [ -f "$CONFIG_PATH" ]; then
  echo "Config file already exists at $CONFIG_PATH, keeping existing configuration."
else
  echo "Copying default config to $CONFIG_PATH..."
  sudo cp "$PACKAGE_DIR/default_config.toml" "$CONFIG_PATH"
fi

echo "Copying systemd service files..."
sudo cp "$SCRIPT_DIR/service_files/cooled-slit-controller.service" /etc/systemd/system/
sudo cp "$SCRIPT_DIR/service_files/cooled-slit-controller-restart.timer" /etc/systemd/system/
sudo cp "$SCRIPT_DIR/service_files/cooled-slit-controller-restart.service" /etc/systemd/system/

echo "Reloading systemd..."
sudo systemctl daemon-reload

echo "Enabling cooled-slit-controller service to start on boot..."
sudo systemctl enable cooled-slit-controller.service
echo "Starting cooled-slit-controller service..."
sudo systemctl start cooled-slit-controller.service

echo "Enabling and starting cooled-slit-controller restart timer..."
sudo systemctl enable cooled-slit-controller-restart.timer
sudo systemctl start cooled-slit-controller-restart.timer


echo ""
echo "Installation complete!"
echo ""
echo "To start the service now, run:"
echo "  sudo systemctl start cooled-slit-controller.service"
echo ""
echo "To check the service status, run:"
echo "  sudo systemctl status cooled-slit-controller.service"
echo ""
echo "To view logs, run:"
echo "  sudo journalctl -u cooled-slit-controller.service"
echo "  or check log files in /var/log/cooled_slit_controller/"
echo ""
echo "The service is configured to automatically restart every 10 minutes."
