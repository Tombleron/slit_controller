# Slit Controller

A system for controlling four motors with encoder feedback, designed for slit control in optical systems.

## Project Overview

- **rf256**: Library for communicating with RF256 linear encoder
- **standa**: Library for controlling Standa motor controllers
- **slit_controller**: Main application that combines these libraries to control a multi-axis slit system

## Architecture

The system is designed with a multi-threaded architecture:

```
+-----------------------------+
| 1. Communication Thread     |  <-- Async, handles client commands
| 2. Controller Thread        |  <-- Owns MultiAxis, executes actions
| 3. State Monitor Thread     |  <-- Polls controller for positions/states
+-----------------------------+
           Shared via channels
```

- **Communication Layer**: Handles Unix domain socket connections, parses commands, and sends them to the controller
- **Controller**: Owns the MultiAxis controller, executes actions like move, stop, etc.
- **State Monitor**: Polls the controller for positions and states of axes, makes them available to clients

## Client Communication

Clients can connect to the Unix domain socket and send commands in the following format:

- `move:{axis}:{position}` - Move an axis to a specific position
- `stop:{axis}` - Stop movement of an axis
- `get:{axis}:{property}` - Get a property of an axis
- `set:{axis}:{property}:{value}` - Set a property of an axis

Where:
- `{axis}` is the axis index (0-3)
- `{property}` can be: position, state, velocity, acceleration, deceleration, or position_window
- `{value}` is the new value for the property

## Client Command Examples

You can communicate with the slit controller using standard Unix tools like `nc` (netcat) or `socat`:

### Using netcat (nc)

```bash
$ nc -U /tmp/slit_controller.sock
get:0:position
move:1:10.5
stop:2
```

### Using socat

```bash
$ socat - UNIX-CONNECT:/tmp/slit_controller.sock
get:0:position
move:1:10.5
stop:2
```
