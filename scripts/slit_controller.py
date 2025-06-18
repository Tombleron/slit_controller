import socket
import time
import re
from typing import Tuple, Dict, Any, Union, Optional, List
from dataclasses import dataclass
from enum import Enum, auto
import logging

from sardana import State
from sardana.pool.controller import Description, MotorController, Type


class SlitControllerError(Exception):
    """Exception raised for errors in the slit controller interaction."""
    pass

@dataclass
class StandaState:
    """Representation of an axis state."""
    state: str  # 'On', 'Moving', or 'Fault'
    limit: str  # 'Upper', 'Lower', 'Both', or 'None'

    @classmethod
    def from_response(cls, response: str) -> 'StandaState':
        # Parse response like "State: (Moving, None)"
        match = re.search(r'\((\w+), (\w+)\)', response)
        if match:
            return cls(state=match.group(1), limit=match.group(2))
        raise SlitControllerError(f"Failed to parse state from response: {response}")

class Slit:
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
            # Clear any existing data in the socket buffer before sending a new command
            self.socket.settimeout(0.001)
            try:
                while True:
                    data = self.socket.recv(1024)
                    if not data:
                        break
            except socket.timeout:
                pass
            self.socket.settimeout(None)  # Reset to blocking mode
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
        match = re.search(r'([-+]?\d*\.\d+|\d+)', response)
        if match:
            return float(match.group(1))
        raise SlitControllerError(f"Failed to parse position from response: {response}")
    def get_state(self, axis: int) -> StandaState:
        """Get the current state of an axis."""
        response = self._send_command(f"get:{axis}:state")
        return StandaState.from_response(response)

    def get_temperature(self, axis: int) -> int:
        """Get the temperature of an axis."""
        response = self._send_command(f"get:{axis}:temperature")
        match = re.search(r'(\d+)', response)
        if match:
            return int(match.group(1))
        raise SlitControllerError(f"Failed to parse temperature from response: {response}")

    def get_velocity(self, axis: int) -> int:
        """Get the velocity setting of an axis."""
        response = self._send_command(f"get:{axis}:velocity")
        match = re.search(r'(\d+)', response)
        if match:
            return int(match.group(1))
        raise SlitControllerError(f"Failed to parse velocity from response: {response}")

    def get_acceleration(self, axis: int) -> int:
        """Get the acceleration setting of an axis."""
        response = self._send_command(f"get:{axis}:acceleration")
        match = re.search(r'(\d+)', response)
        if match:
            return int(match.group(1))
        raise SlitControllerError(f"Failed to parse acceleration from response: {response}")

    def get_deceleration(self, axis: int) -> int:
        """Get the deceleration setting of an axis."""
        response = self._send_command(f"get:{axis}:deceleration")
        match = re.search(r'(\d+)', response)
        if match:
            return int(match.group(1))
        raise SlitControllerError(f"Failed to parse deceleration from response: {response}")

    def get_position_window(self, axis: int) -> float:
        """Get the position window setting of an axis."""
        response = self._send_command(f"get:{axis}:position_window")
        match = re.search(r'([-+]?\d*\.\d+|\d+)', response)
        if match:
            return float(match.group(1))
        raise SlitControllerError(f"Failed to parse position window from response: {response}")

    def get_time_limit(self, axis: int) -> Optional[float]:
        """Get the time limit for an axis."""
        response = self._send_command(f"get:{axis}:time_limit")
        match = re.search(r'([-+]?\d*\.\d+|\d+)', response)
        if match:
            return float(match.group(1))
        raise SlitControllerError(f"Failed to parse time limit from response: {response}")

    def is_moving(self, axis: int) -> bool:
        """Check if an axis is currently moving."""
        response = self._send_command(f"get:{axis}:is_moving")
        match = re.search(r'(true|false)', response)
        if match:
            return match.group(1).lower() == 'true'
        raise SlitControllerError(f"Failed to parse is_moving from response: {response}")



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

    def set_time_limit(self, axis: int, time_limit: int) -> None:
        """Set the time limit for an axis."""
        response = self._send_command(f"set:{axis}:time_limit:{time_limit}")
        if response != "OK":
            raise SlitControllerError(f"Set time limit command failed: {response}")

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



class SlitController(MotorController):
    ctrl_properties = {
    }

    axis_attributes = {
        "LowerLimit": {Type: float, Description: "Нижний предел оси, мм"},
        "UpperLimit": {Type: float, Description: "Верхний предел оси, мм"},

        "Acceleration": {Type: float, Description: "Ускорение оси, шаг/с^2"},
        "Deceleration": {Type: float, Description: "Замедление оси, шаг/с^2"},
        "Velocity": {Type: float, Description: "Скорость оси, шаг/с"},

        "PositionWindow": {Type: float, Description: "Окно допустимой погрешности, мм"},
        "TimeLimit": {Type: float, Description: "Время выполнения движения, с"},
        "Temperature": {Type: int, Description: "Температура ножа, °C"},
    }

    gender = "Motor"
    model = "Блок щелей неохлаждаемый"


    def __init__(self, inst, props, *args, **kwargs):
        super().__init__(inst, props, *args, **kwargs)
        self.logger = logging.getLogger(f"{__name__}.SlitController")

        self.logger.info("Initializing SlitController")
        self.controller = Slit()
        self.logger.info("SlitController initialized successfully")

        # default limits for each axis
        # they will be overridden, since axis properties are Memorized
        self.limits = {
            0: (0.0, 100.0),
            1: (0.0, 100.0),
            2: (0.0, 100.0),
            3: (0.0, 100.0)
        }

    def AddDevice(self, axis: int):
        if axis < 0 or axis >= 4:
            raise SlitControllerError(f"Invalid axis number: {axis}. Must be between 0 and 3.")

    def DeleteDevice(self, axis: int):
        if axis < 0 or axis >= 4:
            raise SlitControllerError(f"Invalid axis number: {axis}. Must be between 0 and 3.")

    def StateOne(self, axis: int):
        state = self.controller.get_state(axis)
        is_moving = self.controller.is_moving(axis)

        if state.limit == "Upper":
            limits = MotorController.UpperLimitSwitch
        elif state.limit == "Lower":
            limits = MotorController.LowerLimitSwitch
        elif state.limit == "Both":
            limits = MotorController.UpperLimitSwitch | MotorController.LowerLimitSwitch
        else:
            limits = MotorController.NoLimitSwitch

        if state.state == "On":
            state_value = State.On
        elif state.state == "Moving":
            state_value = State.Moving
        elif state.state == "Fault":
            state_value = State.Fault
        else:
            state_value = State.Unknown

        if is_moving:
            return State.Moving, f"Controller is moving, motors state is: {state.state}", limits
        else:
            return state_value, "", limits

    def ReadOne(self, axis: int) -> Any:
        return self.controller.get_position(axis)

    def PreStartOne(self, axis: int, value: float) -> bool:
        if value < self.limits[axis][0] or value > self.limits[axis][1]:
            raise SlitControllerError(f"Position {value} out of limits for axis {axis}: {self.limits[axis]}")

        return True

    def StartOne(self, axis: int, value: float):
        self.controller.move(axis, value)

    def StopOne(self, axis: int):
        self.controller.stop(axis)

    def GetAxisPar(self, axis: int, parameter):
        name: str = parameter.lower()

        if name == "acceleration":
            return self.controller.get_acceleration(axis)
        elif name == "deceleration":
            return self.controller.get_deceleration(axis)
        elif name == "velocity":
            return self.controller.get_velocity(axis)
        else:
            raise ValueError(f"Unknown parameter {parameter}")

    def SetAxisPar(self, axis: int, parameter, value):
        name: str = parameter.lower()

        if name == "acceleration":
            self.controller.set_acceleration(axis, value)
        elif name == "deceleration":
            self.controller.set_deceleration(axis, value)
        elif name == "velocity":
            self.controller.set_velocity(axis, value)
        else:
            raise ValueError(f"Unknown parameter {parameter}")

    def GetAxisExtraPar(self, axis: int, parameter):
        name: str = parameter.lower()

        if name == "positionwindow":
            return self.controller.get_position_window(axis)
        elif name == "timelimit":
            return self.controller.get_time_limit(axis)
        elif name == "temperature":
            return self.controller.get_temperature(axis)
        elif name == "lowerlimit":
            return self.limits[axis][0]
        elif name == "upperlimit":
            return self.limits[axis][1]
        else:
            raise ValueError(f"Unknown extra parameter {parameter}")

    def SetAxisExtraPar(self, axis: int, parameter, value):
        name: str = parameter.lower()

        if name == "positionwindow":
            self.controller.set_position_window(axis, value)
        elif name == "timelimit":
            self.controller.set_time_limit(axis, value)
        elif name == "lowerlimit":
            self.limits[axis] = (value, self.limits[axis][1])
        elif name == "upperlimit":
            self.limits[axis] = (self.limits[axis][0], value)
        else:
            raise ValueError(f"Unknown extra parameter {parameter}")


def table(controller):
    import time
    import datetime
    from prettytable import PrettyTable


    try:
        while True:
            # Create a table with headers
            table = PrettyTable()
            table.field_names = ["Axis", "Position", "State", "Moving", "Limit", "Error"]

            # Get data for all axes
            for axis in range(0, 4):
                try:
                    position = controller.get_position(axis)
                    state = controller.get_state(axis)
                    is_moving = controller.is_moving(axis)
                    table.add_row([
                        axis,
                        f"{position:.4f}",
                        state.state,
                        is_moving,
                        state.limit,
                        ""
                    ]);
                except SlitControllerError as e:
                    table.add_row([axis, "N/A", "N/A", "N/A", "N/A", str(e)])

            # Clear screen and print table
            print("\033c", end="")  # Clear terminal
            # Show current time
            current_time = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
            print(f"Current time: {current_time}")
            print(table)
            print("Press Ctrl+C to exit")
            time.sleep(0.1)

    except KeyboardInterrupt:
        print("\nMonitoring stopped by user")


if __name__ == "__main__":
    with Slit() as controller:
        table(controller)
