import sys
import time
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
                             QLabel, QPushButton, QDoubleSpinBox, QGroupBox, QGridLayout,
                             QSlider, QComboBox, QMessageBox, QTabWidget, QSplitter,
                             QTableWidget, QTableWidgetItem, QHeaderView, QSpinBox, QStatusBar)
from PyQt5.QtCore import Qt, QTimer, pyqtSlot
from PyQt5.QtGui import QFont, QColor
import pyqtgraph as pg

from slit_controller import Slit, SlitControllerError, StandaState #type: ignore
from typing import Dict

class AxisControlWidget(QGroupBox):
    """Widget for controlling a single motor axis."""

    def __init__(self, axis_num, controller, parent=None):
        super().__init__(f"Axis {axis_num} Control", parent)
        self.axis_num = axis_num
        self.controller = controller
        self.virtual_zero_offset = 0.0  # Virtual zero offset

        # Create layout
        layout = QGridLayout()
        self.setLayout(layout)

        # Current position display
        layout.addWidget(QLabel("Current Position:"), 0, 0)
        self.position_label = QLabel("---")
        self.position_label.setFont(QFont("Monospace", 12, QFont.Bold))
        layout.addWidget(self.position_label, 0, 1)

        # Virtual position display (relative to virtual zero)
        layout.addWidget(QLabel("Virtual Position:"), 1, 0)
        self.virtual_position_label = QLabel("---")
        self.virtual_position_label.setFont(QFont("Monospace", 12, QFont.Bold))
        layout.addWidget(self.virtual_position_label, 1, 1)

        # Distance to target display
        layout.addWidget(QLabel("Distance to Target:"), 1, 2)
        self.distance_label = QLabel("---")
        self.distance_label.setFont(QFont("Monospace", 12, QFont.Bold))
        layout.addWidget(self.distance_label, 1, 3)

        # State display
        layout.addWidget(QLabel("State:"), 2, 0)
        self.state_label = QLabel("---")
        layout.addWidget(self.state_label, 2, 1)

        # Moving status display
        layout.addWidget(QLabel("Moving:"), 2, 2)
        self.moving_label = QLabel("---")
        self.moving_label.setFont(QFont("Monospace", 10, QFont.Bold))
        layout.addWidget(self.moving_label, 2, 3)

        # Temperature display
        layout.addWidget(QLabel("Temperature:"), 3, 0)
        self.temperature_label = QLabel("---")
        self.temperature_label.setFont(QFont("Monospace", 10, QFont.Bold))
        layout.addWidget(self.temperature_label, 3, 1)

        # Target position input
        layout.addWidget(QLabel("Target Position:"), 4, 0)
        self.target_position = QDoubleSpinBox()
        self.target_position.setRange(-1000, 1000)
        self.target_position.setDecimals(4)
        self.target_position.setSingleStep(0.1)
        layout.addWidget(self.target_position, 4, 1)

        # Mode selection for absolute or relative to virtual zero
        self.position_mode = QComboBox()
        self.position_mode.addItems(["Absolute", "Virtual"])
        self.position_mode.currentIndexChanged.connect(self.update_target_mode)
        layout.addWidget(self.position_mode, 4, 2)

        # Velocity control
        layout.addWidget(QLabel("Velocity:"), 5, 0)
        self.velocity_spinbox = QSpinBox()
        self.velocity_spinbox.setRange(1, 5000)
        layout.addWidget(self.velocity_spinbox, 5, 1)

        # Acceleration control
        layout.addWidget(QLabel("Acceleration:"), 6, 0)
        self.accel_spinbox = QSpinBox()
        self.accel_spinbox.setRange(1, 10000)
        layout.addWidget(self.accel_spinbox, 6, 1)

        # Deceleration control
        layout.addWidget(QLabel("Deceleration:"), 7, 0)
        self.decel_spinbox = QSpinBox()
        self.decel_spinbox.setRange(1, 10000)
        layout.addWidget(self.decel_spinbox, 7, 1)

        # Position window
        layout.addWidget(QLabel("Position Window:"), 8, 0)
        self.pos_window_spinbox = QDoubleSpinBox()
        self.pos_window_spinbox.setRange(0.0001, 1.0)
        self.pos_window_spinbox.setDecimals(4)
        self.pos_window_spinbox.setSingleStep(0.0001)
        layout.addWidget(self.pos_window_spinbox, 8, 1)

        # Time limit
        layout.addWidget(QLabel("Time Limit (s):"), 8, 2)
        self.time_limit_spinbox = QDoubleSpinBox()
        self.time_limit_spinbox.setRange(0.1, 300.0)
        self.time_limit_spinbox.setDecimals(1)
        self.time_limit_spinbox.setSingleStep(1.0)
        self.time_limit_spinbox.setValue(30.0)  # Default 30 seconds
        layout.addWidget(self.time_limit_spinbox, 8, 3)

        # Virtual Zero controls
        zero_layout = QHBoxLayout()

        # Set Virtual Zero button
        self.set_zero_button = QPushButton("Set Virtual Zero")
        self.set_zero_button.clicked.connect(self.set_virtual_zero)
        zero_layout.addWidget(self.set_zero_button)

        # Reset Virtual Zero button
        self.reset_zero_button = QPushButton("Reset Virtual Zero")
        self.reset_zero_button.clicked.connect(self.reset_virtual_zero)
        zero_layout.addWidget(self.reset_zero_button)

        layout.addLayout(zero_layout, 9, 0, 1, 3)

        # Buttons
        button_layout = QHBoxLayout()

        # Move button
        self.move_button = QPushButton("Move")
        self.move_button.clicked.connect(self.move_axis)
        button_layout.addWidget(self.move_button)

        # Stop button
        self.stop_button = QPushButton("Stop")
        self.stop_button.clicked.connect(self.stop_axis)
        self.stop_button.setStyleSheet("background-color: #ff6666;")
        button_layout.addWidget(self.stop_button)

        # Apply settings button
        self.apply_button = QPushButton("Apply Settings")
        self.apply_button.clicked.connect(self.apply_settings)
        button_layout.addWidget(self.apply_button)

        layout.addLayout(button_layout, 10, 0, 1, 3)

        # Initialize with current values
        self.update_display()
        self.load_settings()

    def update_display(self):
        """Update the displayed information about the axis."""
        # Get position data
        try:
            position = self.controller.get_position(self.axis_num)
            self.position_label.setText(f"{position:.4f}")

            # Calculate and display virtual position
            virtual_position = position - self.virtual_zero_offset
            self.virtual_position_label.setText(f"{virtual_position:.4f}")

            # Calculate and display distance to target
            target = self.target_position.value()
            # Adjust target value if in virtual mode
            if self.position_mode.currentIndex() == 1:  # Virtual mode
                target = target + self.virtual_zero_offset
            distance = target - position
            self.distance_label.setText(f"{distance:.4f}")

            # Set color based on distance
            if abs(distance) < 0.0001:
                self.distance_label.setStyleSheet("color: #00aa00; font-weight: bold;")  # Green if at target
            elif abs(distance) < 0.001:
                self.distance_label.setStyleSheet("color: #0000ff; font-weight: bold;")  # Blue if close
            else:
                self.distance_label.setStyleSheet("color: #000000; font-weight: bold;")  # Black if far
        except SlitControllerError as e:
            self.position_label.setText(f"Error")
            self.virtual_position_label.setText(f"Error")
            self.distance_label.setText(f"Error")
            # Don't update state label with position errors

        # Get state data
        try:
            state = self.controller.get_state(self.axis_num)
            is_moving = self.controller.is_moving(self.axis_num)

            # Update state label with color indication
            state_text = f"{state.state}"
            if state.limit != "None":
                state_text += f" (Limit: {state.limit})"

            self.state_label.setText(state_text)

            # Update moving status
            self.moving_label.setText(str(is_moving))
            if is_moving:
                self.moving_label.setStyleSheet("color: #0000ff; font-weight: bold;")
            else:
                self.moving_label.setStyleSheet("color: #00aa00; font-weight: bold;")

            # Set colors based on state
            if state.state == "Fault":
                self.state_label.setStyleSheet("color: #ff0000; font-weight: bold;")
            elif state.state == "Moving":
                self.state_label.setStyleSheet("color: #0000ff; font-weight: bold;")
            elif state.limit != "None":
                self.state_label.setStyleSheet("color: #ff9900; font-weight: bold;")
            else:
                self.state_label.setStyleSheet("color: #00aa00; font-weight: bold;")

            # Update button states
            self.move_button.setEnabled(state.state != "Fault")
        except SlitControllerError as e:
            self.state_label.setText(f"Error: {str(e)}")
            self.moving_label.setText("Error")
            self.state_label.setStyleSheet("color: #ff0000; font-weight: bold;")

        # Get temperature data
        try:
            temperature = self.controller.get_temperature(self.axis_num)
            self.temperature_label.setText(f"{temperature}°C")

            # Set color based on temperature
            if temperature >= 60:
                self.temperature_label.setStyleSheet("color: #ffffff; background-color: #ff0000; font-weight: bold;")  # White on red for high temperature
            elif temperature >= 45:
                self.temperature_label.setStyleSheet("color: #000000; background-color: #ff9600; font-weight: bold;")  # Black on orange for warning
            elif temperature >= 35:
                self.temperature_label.setStyleSheet("color: #000000; background-color: #ffff96; font-weight: bold;")  # Black on light yellow for elevated
            else:
                self.temperature_label.setStyleSheet("color: #000000; font-weight: bold;")  # Default
        except SlitControllerError as e:
            self.temperature_label.setText("Error")
            self.temperature_label.setStyleSheet("color: #ff0000; font-weight: bold;")
            # Keep the error in the temperature field only

    def load_settings(self):
        """Load current settings from the controller."""
        try:
            # Get current settings
            velocity = self.controller.get_velocity(self.axis_num)
            accel = self.controller.get_acceleration(self.axis_num)
            decel = self.controller.get_deceleration(self.axis_num)
            pos_window = self.controller.get_position_window(self.axis_num)
            time_limit = self.controller.get_time_limit(self.axis_num)

            # Update UI
            self.velocity_spinbox.setValue(velocity)
            self.accel_spinbox.setValue(accel)
            self.decel_spinbox.setValue(decel)
            self.pos_window_spinbox.setValue(pos_window)
            if time_limit is not None:
                self.time_limit_spinbox.setValue(time_limit)

            # Set target position to current position
            position = self.controller.get_position(self.axis_num)
            self.target_position.setValue(position)

        except SlitControllerError as e:
            QMessageBox.warning(self, "Error", f"Failed to load settings: {str(e)}")

    def set_virtual_zero(self):
        """Set the current position as virtual zero."""
        try:
            current_position = self.controller.get_position(self.axis_num)
            self.virtual_zero_offset = current_position
            QMessageBox.information(self, "Virtual Zero Set",
                                   f"Virtual zero for Axis {self.axis_num} set to {current_position:.4f}")
            self.update_display()
        except SlitControllerError as e:
            QMessageBox.warning(self, "Virtual Zero Error", str(e))

    def reset_virtual_zero(self):
        """Reset the virtual zero to absolute zero."""
        self.virtual_zero_offset = 0.0
        QMessageBox.information(self, "Virtual Zero Reset",
                               f"Virtual zero for Axis {self.axis_num} reset to absolute zero")
        self.update_display()

    def update_target_mode(self):
        """Update the target input based on the selected mode."""
        try:
            current_value = self.target_position.value()
            if self.position_mode.currentIndex() == 0:  # Absolute mode
                # If switching from virtual to absolute, convert the value
                if hasattr(self, '_last_mode') and self._last_mode == 1:
                    self.target_position.setValue(current_value + self.virtual_zero_offset)
            else:  # Virtual mode
                # If switching from absolute to virtual, convert the value
                if not hasattr(self, '_last_mode') or self._last_mode == 0:
                    self.target_position.setValue(current_value - self.virtual_zero_offset)

            self._last_mode = self.position_mode.currentIndex()
        except SlitControllerError as e:
            QMessageBox.warning(self, "Mode Change Error", str(e))

    def move_axis(self):
        """Move the axis to the target position."""
        try:
            target = self.target_position.value()

            # If in virtual mode, convert to absolute position
            if self.position_mode.currentIndex() == 1:  # Virtual mode
                target = target + self.virtual_zero_offset


            self.controller.move(self.axis_num, target)

            # Immediate update to show movement started
            self.update_display()
        except SlitControllerError as e:
            QMessageBox.warning(self, "Move Error", str(e))

    def stop_axis(self):
        """Stop the axis movement."""
        try:
            self.controller.stop(self.axis_num)
            self.update_display()
        except SlitControllerError as e:
            QMessageBox.warning(self, "Stop Error", str(e))

    def apply_settings(self):
        """Apply motion settings to the controller."""
        try:
            velocity = self.velocity_spinbox.value()
            accel = self.accel_spinbox.value()
            decel = self.decel_spinbox.value()
            pos_window = self.pos_window_spinbox.value()
            time_limit = self.time_limit_spinbox.value()

            self.controller.set_velocity(self.axis_num, velocity)
            self.controller.set_acceleration(self.axis_num, accel)
            self.controller.set_deceleration(self.axis_num, decel)
            self.controller.set_position_window(self.axis_num, pos_window)
            self.controller.set_time_limit(self.axis_num, int(time_limit))

            QMessageBox.information(self, "Settings Applied",
                                   f"Settings for Axis {self.axis_num} applied successfully.")

        except SlitControllerError as e:
            QMessageBox.warning(self, "Settings Error", str(e))


class AxisMonitorWidget(QTableWidget):
    """Widget for monitoring all axes in a table view."""

    def __init__(self, controller, parent=None):
        super().__init__(parent)
        self.controller = controller
        self.virtual_zero_offsets = {}  # Dictionary to store virtual zero offsets by axis

        # Set up table
        self.setColumnCount(9)  # Added columns for virtual position, distance to target, and temperature
        self.setHorizontalHeaderLabels(["Axis", "Position", "Virtual Position", "Distance to Target", "State", "Moving", "Limit", "Temperature", "Error"])
        self.setRowCount(4)  # Assuming 4 axes (0-3)

        # Set table properties
        header = self.horizontalHeader()
        header.setSectionResizeMode(QHeaderView.Stretch)
        self.setEditTriggers(QTableWidget.NoEditTriggers)
        self.setAlternatingRowColors(True)

        # Initial update
        self.update_table()

    def update_table(self):
        """Update the table with current axis information."""
        for axis in range(4):  # Assuming 4 axes (0-3)
            # Set the axis number for each row (this never fails)
            self.setItem(axis, 0, QTableWidgetItem(str(axis)))

            # Get position data
            try:
                position = self.controller.get_position(axis)
                self.setItem(axis, 1, QTableWidgetItem(f"{position:.4f}"))

                # Calculate virtual position
                virtual_zero = self.virtual_zero_offsets.get(axis, 0.0)
                virtual_position = position - virtual_zero
                self.setItem(axis, 2, QTableWidgetItem(f"{virtual_position:.4f}"))

                # Calculate distance to target (if there's a target)
                try:
                    # Find corresponding axis control widget
                    target_value = None
                    for control in self.parent().findChildren(AxisControlWidget):
                        if control.axis_num == axis:
                            target_value = control.target_position.value()
                            # Adjust target value if in virtual mode
                            if control.position_mode.currentIndex() == 1:  # Virtual mode
                                target_value += control.virtual_zero_offset
                            break

                    if target_value is not None:
                        distance = target_value - position
                        distance_str = f"{distance:.4f}"
                        self.setItem(axis, 3, QTableWidgetItem(distance_str))

                        # Set color for distance cell based on how close to target
                        try:
                            distance_val = float(distance_str)
                            if abs(distance_val) < 0.0001:
                                # At target
                                self.item(axis, 3).setBackground(QColor(200, 255, 200))  # Green
                            elif abs(distance_val) < 0.001:
                                # Close to target
                                self.item(axis, 3).setBackground(QColor(200, 200, 255))  # Blue
                        except:
                            pass
                    else:
                        self.setItem(axis, 3, QTableWidgetItem("---"))
                except:
                    self.setItem(axis, 3, QTableWidgetItem("---"))
            except SlitControllerError as e:
                self.setItem(axis, 1, QTableWidgetItem("Error"))
                self.setItem(axis, 2, QTableWidgetItem("Error"))
                self.setItem(axis, 3, QTableWidgetItem("Error"))
                # Display the error in the error column at the end
                self.setItem(axis, 8, QTableWidgetItem(f"Position error: {str(e)}"))

            # Get state data
            try:
                state = self.controller.get_state(axis)
                is_moving = self.controller.is_moving(axis)

                self.setItem(axis, 4, QTableWidgetItem(state.state))
                self.setItem(axis, 5, QTableWidgetItem(str(is_moving)))
                self.setItem(axis, 6, QTableWidgetItem(state.limit))

                # Set colors based on state
                if state.state == "Fault":
                    self.item(axis, 4).setBackground(QColor(255, 200, 200))
                elif state.state == "Moving":
                    self.item(axis, 4).setBackground(QColor(200, 200, 255))
                elif state.limit != "None":
                    self.item(axis, 4).setBackground(QColor(255, 230, 180))
                else:
                    self.item(axis, 4).setBackground(QColor(200, 255, 200))
            except SlitControllerError as e:
                self.setItem(axis, 4, QTableWidgetItem("Error"))
                self.setItem(axis, 5, QTableWidgetItem("Error"))
                self.setItem(axis, 6, QTableWidgetItem("Error"))
                self.item(axis, 4).setBackground(QColor(255, 200, 200))
                # Set or append to the error message in the error column
                error_cell = self.item(axis, 8)
                if error_cell and error_cell.text():
                    self.setItem(axis, 8, QTableWidgetItem(f"{error_cell.text()}, State error: {str(e)}"))
                else:
                    self.setItem(axis, 8, QTableWidgetItem(f"State error: {str(e)}"))

            # Get temperature data
            try:
                temperature = self.controller.get_temperature(axis)
                temp_item = QTableWidgetItem(f"{temperature}°C")
                self.setItem(axis, 7, temp_item)

                # Set background color based on temperature
                if temperature >= 60:
                    temp_item.setBackground(QColor(255, 0, 0))  # Red for high temperature
                    temp_item.setForeground(QColor(255, 255, 255))  # White text for readability
                elif temperature >= 45:
                    temp_item.setBackground(QColor(255, 150, 0))  # Orange for warning
                elif temperature >= 35:
                    temp_item.setBackground(QColor(255, 255, 150))  # Light yellow for elevated
            except SlitControllerError as e:
                temp_item = QTableWidgetItem("Error")
                self.setItem(axis, 7, temp_item)
                # Set or append to the error message in the error column
                error_cell = self.item(axis, 8)
                if error_cell and error_cell.text():
                    self.setItem(axis, 8, QTableWidgetItem(f"{error_cell.text()}, Temp error: {str(e)}"))
                else:
                    self.setItem(axis, 8, QTableWidgetItem(f"Temp error: {str(e)}"))


class PositionPlotWidget(QWidget):
    """Widget for plotting position history."""

    def __init__(self, controller, parent=None):
        super().__init__(parent)
        self.controller = controller
        self.history_length = 100  # Number of points to keep in history

        # Initialize position history for each axis
        self.time_data = []
        self.position_data = [[] for _ in range(4)]
        self.start_time = time.time()

        # Create layout
        layout = QVBoxLayout()
        self.setLayout(layout)

        # Create plot
        self.plot_widget = pg.PlotWidget()
        self.plot_widget.setLabel('left', 'Position')
        self.plot_widget.setLabel('bottom', 'Time (s)')
        self.plot_widget.addLegend()
        layout.addWidget(self.plot_widget)

        # Create curves for each axis
        self.curves = []
        colors = ['r', 'g', 'b', 'y']  # Colors for each axis
        for i in range(4):
            curve = self.plot_widget.plot(pen=colors[i], name=f'Axis {i}')
            self.curves.append(curve)

        # Checkbox layout for showing/hiding axes
        checkbox_layout = QHBoxLayout()
        self.checkboxes = []

        for i in range(4):
            checkbox = QComboBox()
            checkbox.addItems([f"Show Axis {i}", f"Hide Axis {i}"])
            checkbox.setCurrentIndex(0)  # Default to showing
            checkbox.currentIndexChanged.connect(self.update_visible_curves)
            checkbox_layout.addWidget(checkbox)
            self.checkboxes.append(checkbox)

        layout.addLayout(checkbox_layout)

    def update_visible_curves(self):
        """Update which curves are visible based on checkboxes."""
        for i, checkbox in enumerate(self.checkboxes):
            visible = checkbox.currentIndex() == 0  # 0 = Show, 1 = Hide
            self.curves[i].setVisible(visible)

    def update_plot(self):
        """Update the position history plot."""
        try:
            # Add current time
            current_time = time.time() - self.start_time
            self.time_data.append(current_time)

            # Limit history length
            if len(self.time_data) > self.history_length:
                self.time_data.pop(0)
                for axis_data in self.position_data:
                    if axis_data:
                        axis_data.pop(0)

            # Get current positions for each axis - handle each independently
            errors = []
            for axis in range(4):
                try:
                    position = self.controller.get_position(axis)
                    self.position_data[axis].append(position)
                    # Ensure all data lists have the same length
                    while len(self.position_data[axis]) < len(self.time_data):
                        self.position_data[axis].insert(0, self.position_data[axis][0] if self.position_data[axis] else 0)
                except SlitControllerError as e:
                    # If there's an error, repeat the last position or use 0
                    last_position = self.position_data[axis][-1] if self.position_data[axis] else 0
                    self.position_data[axis].append(last_position)
                    errors.append(f"Axis {axis} position error: {str(e)}")

            # Update each curve independently - if one fails, the others should still update
            for i, curve in enumerate(self.curves):
                try:
                    if i < len(self.position_data) and self.position_data[i]:
                        curve.setData(self.time_data, self.position_data[i])
                except Exception as curve_error:
                    errors.append(f"Curve {i} update error: {str(curve_error)}")

            # Log any errors that occurred but didn't stop the overall update
            if errors:
                print(f"Plot partial update errors: {'; '.join(errors)}")

        except Exception as e:
            print(f"Plot update critical error: {str(e)}")


class MotorControlApp(QMainWindow):
    """Main application window for motor control."""

    def __init__(self):
        super().__init__()

        # Set up the window
        self.setWindowTitle("Standa Motor Controller")
        self.setGeometry(100, 100, 1200, 800)

        # Store virtual zero offsets
        self.virtual_zero_offsets = {}  # Dictionary to store virtual zero offsets by axis

        # Create status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)

        # Connection status label
        self.connection_status = QLabel("Disconnected")
        self.connection_status.setStyleSheet("color: red; font-weight: bold;")
        self.status_bar.addPermanentWidget(self.connection_status)

        # Set up controller
        self.controller = Slit()
        self.connected = False

        # Create central widget and layout
        central_widget = QWidget()
        self.setCentralWidget(central_widget)
        main_layout = QVBoxLayout(central_widget)

        # Create connection button
        self.connect_button = QPushButton("Connect to Controller")
        self.connect_button.clicked.connect(self.toggle_connection)
        main_layout.addWidget(self.connect_button)

        # Create tab widget
        self.tab_widget = QTabWidget()
        main_layout.addWidget(self.tab_widget)

        # Create individual control tab
        control_tab = QWidget()
        control_layout = QHBoxLayout(control_tab)

        # Create axis control widgets
        self.axis_controls = []
        for i in range(4):  # Assuming 4 axes (0-3)
            axis_control = AxisControlWidget(i, self.controller)
            control_layout.addWidget(axis_control)
            self.axis_controls.append(axis_control)

        self.tab_widget.addTab(control_tab, "Individual Control")

        # Create monitor tab
        monitor_tab = QWidget()
        monitor_layout = QVBoxLayout(monitor_tab)

        # Create monitor table
        self.monitor_table = AxisMonitorWidget(self.controller)
        monitor_layout.addWidget(self.monitor_table)

        # Add virtual zero controls to monitor tab
        virtual_zero_layout = QHBoxLayout()

        # Button to save virtual zeros for all axes
        self.save_zero_config_button = QPushButton("Save Virtual Zero Configuration")
        self.save_zero_config_button.clicked.connect(self.save_virtual_zero_config)
        virtual_zero_layout.addWidget(self.save_zero_config_button)

        # Button to load virtual zero configuration
        self.load_zero_config_button = QPushButton("Load Virtual Zero Configuration")
        self.load_zero_config_button.clicked.connect(self.load_virtual_zero_config)
        virtual_zero_layout.addWidget(self.load_zero_config_button)

        # Button to reset all virtual zeros
        self.reset_all_zeros_button = QPushButton("Reset All Virtual Zeros")
        self.reset_all_zeros_button.clicked.connect(self.reset_all_virtual_zeros)
        virtual_zero_layout.addWidget(self.reset_all_zeros_button)

        monitor_layout.addLayout(virtual_zero_layout)

        # Create position plot
        self.position_plot = PositionPlotWidget(self.controller)
        monitor_layout.addWidget(self.position_plot)

        self.tab_widget.addTab(monitor_tab, "Monitor")

        # Create update timer
        self.update_timer = QTimer()
        self.update_timer.timeout.connect(self.update_displays)
        self.update_timer.setInterval(100)  # 100ms update interval

        # Disable controls until connected
        self.set_controls_enabled(False)

    def toggle_connection(self):
        """Connect to or disconnect from the controller."""
        if not self.connected:
            try:
                self.controller.connect()
                self.connected = True
                self.connect_button.setText("Disconnect from Controller")
                self.connection_status.setText("Connected")
                self.connection_status.setStyleSheet("color: green; font-weight: bold;")
                self.set_controls_enabled(True)
                self.update_timer.start()
                self.status_bar.showMessage("Connected to controller", 3000)
            except SlitControllerError as e:
                QMessageBox.critical(self, "Connection Error",
                                    f"Failed to connect to controller: {str(e)}")
        else:
            try:
                self.update_timer.stop()
                self.controller.disconnect()
                self.connected = False
                self.connect_button.setText("Connect to Controller")
                self.connection_status.setText("Disconnected")
                self.connection_status.setStyleSheet("color: red; font-weight: bold;")
                self.set_controls_enabled(False)
                self.status_bar.showMessage("Disconnected from controller", 3000)
            except SlitControllerError as e:
                QMessageBox.warning(self, "Disconnection Error",
                                  f"Error during disconnection: {str(e)}")

    def set_controls_enabled(self, enabled):
        """Enable or disable all control widgets."""
        for axis_control in self.axis_controls:
            axis_control.setEnabled(enabled)

    def save_virtual_zero_config(self):
        """Save the current virtual zero configuration."""
        try:
            # Collect virtual zero values from all axis controls
            zero_config = {}
            for axis_control in self.axis_controls:
                zero_config[axis_control.axis_num] = axis_control.virtual_zero_offset

            # Use a JSON file to save the configuration
            import json
            import os

            config_dir = os.path.expanduser("~/.config/standa_gui")
            os.makedirs(config_dir, exist_ok=True)

            config_path = os.path.join(config_dir, "virtual_zero.json")
            with open(config_path, 'w') as f:
                json.dump(zero_config, f)

            self.status_bar.showMessage(f"Virtual zero configuration saved to {config_path}", 3000)
        except Exception as e:
            QMessageBox.warning(self, "Save Configuration Error", str(e))

    def load_virtual_zero_config(self):
        """Load a saved virtual zero configuration."""
        try:
            import json
            import os

            config_path = os.path.join(os.path.expanduser("~/.config/standa_gui"), "virtual_zero.json")

            if not os.path.exists(config_path):
                QMessageBox.information(self, "Configuration Not Found",
                                      "No saved virtual zero configuration found.")
                return

            with open(config_path, 'r') as f:
                zero_config = json.load(f)

            # Apply loaded configuration to axis controls
            for axis_control in self.axis_controls:
                if str(axis_control.axis_num) in zero_config:
                    axis_control.virtual_zero_offset = float(zero_config[str(axis_control.axis_num)])

            # Update monitor table with new offsets
            for axis, offset in zero_config.items():
                self.monitor_table.virtual_zero_offsets[int(axis)] = float(offset)

            # Refresh displays
            for axis_control in self.axis_controls:
                axis_control.update_display()
            self.monitor_table.update_table()

            self.status_bar.showMessage("Virtual zero configuration loaded", 3000)
        except Exception as e:
            QMessageBox.warning(self, "Load Configuration Error", str(e))

    def reset_all_virtual_zeros(self):
        """Reset all virtual zeros to absolute zero."""
        for axis_control in self.axis_controls:
            axis_control.virtual_zero_offset = 0.0
            axis_control.update_display()

        self.monitor_table.virtual_zero_offsets = {}
        self.monitor_table.update_table()

        self.status_bar.showMessage("All virtual zeros reset to absolute zero", 3000)

    def update_displays(self):
        """Update all display widgets."""
        if self.connected:
            errors = []

            # Update axis controls - continue even if some fail
            for axis_control in self.axis_controls:
                try:
                    axis_control.update_display()
                    # Transfer virtual zero offsets to the monitor table
                    self.monitor_table.virtual_zero_offsets[axis_control.axis_num] = axis_control.virtual_zero_offset
                except Exception as e:
                    errors.append(f"Axis {axis_control.axis_num} update error: {str(e)}")

            # Update monitor table - attempt even if axis controls had issues
            try:
                self.monitor_table.update_table()
            except Exception as e:
                errors.append(f"Monitor table update error: {str(e)}")

            # Update position plot - attempt even if other components had issues
            try:
                self.position_plot.update_plot()
            except Exception as e:
                errors.append(f"Position plot update error: {str(e)}")

            # Display errors in status bar if any occurred
            if errors:
                self.status_bar.showMessage(f"Some update errors occurred. See error fields for details.", 3000)

    def closeEvent(self, a0):
        """Handle window close event."""
        if self.connected:
            try:
                self.update_timer.stop()
                self.controller.disconnect()
            except:
                pass
        a0.accept()


if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = MotorControlApp()
    window.show()
    sys.exit(app.exec_())
