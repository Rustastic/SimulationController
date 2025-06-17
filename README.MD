# SimulationController

## Overview
This is the Simulation Controller for the drone-based network developed by the **Rustastic** group.

## Simulation Controller
The **Simulation Controller** serves as the central coordinator of the simulation. It manages communication between all components, handles user commands from the GUI, and ensures that drones, clients, and the server behave coherently throughout the simulation.

### Responsibilities
* **Initialization**: Receives all configuration and state data from the [NetworkInitializer](https://github.com/Rustastic/NetworkInitializer) and prepares the simulation environment.
* **Command Handling**: Listens to commands issued by the [GUI](https://github.com/Rustastic/GUI), interprets them, and forwards appropriate actions to the relevant simulation entities (drones, clients, or server).
* **Event Handling**: Processes events coming from drones, clients, and the server, and appropriately updates the system state or notifies the GUI.
* **Communication Management**: Ensures that messages, updates, and commands are correctly routed between all network participants.
* **Logic & Validation**: Applies internal logic checks and validation rules to maintain a consistent and valid simulation state.

### Features
* Centralized handling of commands and events.
* Real-time coordination between drones, clients, and server.
* Runtime validation to ensure consistency and prevent errors.
* Seamless integration with the GUI for interactive control.
* Modular design, allowing for future feature extensions.