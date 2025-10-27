#!/bin/bash

# Function to check if an axis is moving
check_axis_moving() {
    local axis=$1
    local result

    # Send command to check if axis is moving
    result=$(echo "get:$axis:is_moving" | nc -U /tmp/cooled_slit_controller.sock 2>/dev/null)

    # Check if the result indicates the axis is moving (true)
    if [ "$result" = "true" ]; then
        return 0  # axis is moving
    else
        return 1  # axis is not moving
    fi
}

# Function to check if any of axes 1-4 are moving
check_any_axis_moving() {
    for axis in 1 2 3 4; do
        if check_axis_moving $axis; then
            echo "Axis $axis is currently moving"
            return 0  # at least one axis is moving
        fi
    done
    return 1  # no axes are moving
}

# Main logic
if check_any_axis_moving; then
    echo "One or more axes (1-4) are currently moving. Skipping restart."
    exit 0
else
    echo "No axes (1-4) are moving. Proceeding with restart."
    systemctl restart cooled-slit-controller.service
    echo "Cooled slit controller service restarted."
fi
