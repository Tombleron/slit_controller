#!/usr/bin/env python3
import socket
import time
import re
from typing import Tuple, Dict, Any, Union, Optional, List
from dataclasses import dataclass
from enum import Enum, auto

class SlitControllerError(Exception):
    """Exception raised for errors in the slit controller interaction."""
    pass

class AxisProperty(Enum):
    """Properties that can be retrieved or set for an axis."""
    POSITION = "position"
    STATE = "state"
    VELOCITY = "velocity"
    ACCELERATION = "acceleration"
    DECELERATION = "deceleration"
    POSITION_WINDOW = "position_window"

@dataclass
class State:
    """Representation of an axis state."""
    state: str  # 'On', 'Moving', or 'Fault'
    limit: str  # 'Upper', 'Lower', 'Both', or 'None'

    @classmethod
    def from_response(cls, response: str) -> 'State':
        # Parse response like "State: (Moving, None)"
        match = re.search(r'\((\w+), (\w+)\)', response)
        if match:
            return cls(state=match.group(1), limit=match.group(2))
        raise SlitControllerError(f"Failed to parse state from response: {response}")

class SlitController:
    """Client for interacting with the slit controller via Unix socket."""

    SOCKET_PATH = "/tmp/slit_controller.sock"

    def __init__(self):
        """Initialize the slit controller client."""
        self.socket = None

    def connect(self) -> None:
        """Connect to the slit controller socket."""
        try:
            self.socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            self.socket.connect(self.SOCKET_PATH)
        except (socket.error, FileNotFoundError) as e:
            raise SlitControllerError(f"Failed to connect to socket: {e}")

    def disconnect(self) -> None:
        """Close the connection to the slit controller."""
        if self.socket:
            self.socket.close()
            self.socket = None

    def __enter__(self):
        """Context manager entry."""
        self.connect()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.disconnect()

    def _send_command(self, command: str) -> str:
        """Send a command to the slit controller and return the response."""
        if not self.socket:
            self.connect()

        try:
            self.socket.sendall(command.encode('utf-8')) # type: ignore
            response = self.socket.recv(1024).decode('utf-8').strip() # type: ignore

            if response.startswith("Error:"):
                raise SlitControllerError(response)

            return response
        except socket.error as e:
            self.disconnect()
            raise SlitControllerError(f"Socket error: {e}")

    def move(self, axis: int, position: float) -> None:
        """Move an axis to a specified position."""
        response = self._send_command(f"move:{axis}:{position}")
        if response != "OK":
            raise SlitControllerError(f"Move command failed: {response}")

    def stop(self, axis: int) -> None:
        """Stop movement of an axis."""
        response = self._send_command(f"stop:{axis}")
        if response != "OK":
            raise SlitControllerError(f"Stop command failed: {response}")

    def get_position(self, axis: int) -> float:
        """Get the current position of an axis."""
        response = self._send_command(f"get:{axis}:position")
        match = re.search(r'Position: ([-+]?\d*\.\d+|\d+)', response)
        if match:
            return float(match.group(1))
        raise SlitControllerError(f"Failed to parse position from response: {response}")

    def get_state(self, axis: int) -> State:
        """Get the current state of an axis."""
        response = self._send_command(f"get:{axis}:state")
        return State.from_response(response)

    def get_velocity(self, axis: int) -> int:
        """Get the velocity setting of an axis."""
        response = self._send_command(f"get:{axis}:velocity")
        match = re.search(r'Velocity: (\d+)', response)
        if match:
            return int(match.group(1))
        raise SlitControllerError(f"Failed to parse velocity from response: {response}")

    def get_acceleration(self, axis: int) -> int:
        """Get the acceleration setting of an axis."""
        response = self._send_command(f"get:{axis}:acceleration")
        match = re.search(r'Acceleration: (\d+)', response)
        if match:
            return int(match.group(1))
        raise SlitControllerError(f"Failed to parse acceleration from response: {response}")

    def get_deceleration(self, axis: int) -> int:
        """Get the deceleration setting of an axis."""
        response = self._send_command(f"get:{axis}:deceleration")
        match = re.search(r'Deceleration: (\d+)', response)
        if match:
            return int(match.group(1))
        raise SlitControllerError(f"Failed to parse deceleration from response: {response}")

    def get_position_window(self, axis: int) -> float:
        """Get the position window setting of an axis."""
        response = self._send_command(f"get:{axis}:position_window")
        match = re.search(r'Position Window: ([-+]?\d*\.\d+|\d+)', response)
        if match:
            return float(match.group(1))
        raise SlitControllerError(f"Failed to parse position window from response: {response}")

    def set_velocity(self, axis: int, velocity: int) -> None:
        """Set the velocity for an axis."""
        response = self._send_command(f"set:{axis}:velocity:{velocity}")
        if response != "OK":
            raise SlitControllerError(f"Set velocity command failed: {response}")

    def set_acceleration(self, axis: int, acceleration: int) -> None:
        """Set the acceleration for an axis."""
        response = self._send_command(f"set:{axis}:acceleration:{acceleration}")
        if response != "OK":
            raise SlitControllerError(f"Set acceleration command failed: {response}")

    def set_deceleration(self, axis: int, deceleration: int) -> None:
        """Set the deceleration for an axis."""
        response = self._send_command(f"set:{axis}:deceleration:{deceleration}")
        if response != "OK":
            raise SlitControllerError(f"Set deceleration command failed: {response}")

    def set_position_window(self, axis: int, window: float) -> None:
        """Set the position window for an axis."""
        response = self._send_command(f"set:{axis}:position_window:{window}")
        if response != "OK":
            raise SlitControllerError(f"Set position window command failed: {response}")

    def wait_for_motion_complete(self, axis: int, timeout: float = 30.0, poll_interval: float = 0.1) -> None:
        """Wait for an axis to stop moving, with timeout in seconds."""
        start_time = time.time()
        while time.time() - start_time < timeout:
            state = self.get_state(axis)
            if state.state != 'Moving':
                return
            time.sleep(poll_interval)

        raise SlitControllerError(f"Timeout waiting for axis {axis} to complete movement")

    def move_and_wait(self, axis: int, position: float, timeout: float = 30.0) -> None:
        """Move an axis to a position and wait for completion."""
        self.move(axis, position)
        self.wait_for_motion_complete(axis, timeout)


# Example usage
if __name__ == "__main__":
        with SlitController() as controller:

            import time
            from prettytable import PrettyTable

            try:
                while True:
                    # Create a table with headers
                    table = PrettyTable()
                    table.field_names = ["Axis", "Position", "State", "Limit", "Error"]

                    # Get data for all axes
                    for axis in range(0, 4):
                        try:
                            position = controller.get_position(axis)
                            state = controller.get_state(axis)
                            table.add_row([
                                axis,
                                f"{position:.3f}",
                                state.state,
                                state.limit,
                                ""
                            ])
                        except SlitControllerError as e:
                            table.add_row([axis, "N/A", "N/A", "N/A", str(e)])

                    # Clear screen and print table
                    print("\033c", end="")  # Clear terminal
                    print(table)
                    print("Press Ctrl+C to exit")
                    time.sleep(0.5)

            except KeyboardInterrupt:
                print("\nMonitoring stopped by user")



            # Set parameters
            # controller.set_velocity(0, 1000)
            # controller.set_acceleration(0, 500)
            # controller.set_deceleration(0, 500)

            # Move axis
            # print("Moving axis 0 to position 10.0...")
            # controller.move_and_wait(0, 10.0)

            # Check final position
            # position = controller.get_position(0)
            # print(f"Final position of axis 0: {position}")
