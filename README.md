
# growbe-mainboard

Repository for the growbe-mainboard

[Pi documentation](/docs/pi.md)



### Build the app

```
# Build the docker image
./scripts/docker.sh

# Build the C driver library
# Build for linux pc
./scripts/rust_env.sh make -C ./drivers
# Build for linux arm
TARGET_NAME=armv7-unknown-linux-gnueabihf ./scripts/rust_env.sh make -C ./drivers

# Build the app for local dev / test
./scripts/rust_env.sh cargo vendors
./scripts/rust_env.sh cargo build 
./target/debug/growbe-mainboard ./mainboard_config.json

# Start the debug app with gdb on a remote machine
TARGET_NAME=armv7-unknown-linux-gnueabihf ./scripts/remote_debug.sh . 192.168.50.41 17777
```


### Run application

```bash
./target/debug/growbe-mainboard ./mainboard_config.json

# With all logs
RUST_LOG=growbe_mainboard ./target/debug/growbe-mainboard ./mainboard_config.json
# Log from a module
RUST_LOG=growbe_mainboard::socket=debug ./target/debug/growbe-mainboard ./mainboard_config.json
```

### Configuring for VSCode

#### Plugin 

Use this plugin not the official one. https://rust-analyzer.github.io/manual.html#vs-code

#### Debug

`.vscode/launch.json`
```json
{
	// Use IntelliSense to learn about possible attributes.
	// Hover to view descriptions of existing attributes.
	// For more information, visit: https://go.microsoft.com/fwlink/?linkid=830387
	"version": "0.2.0",
	"configurations": [
  	{
      "name": "C++ Launch",
      "type": "cppdbg",
      "request": "launch",
	  "preLaunchTask": "rust: remote ARM debug setup",
      "program": "${workspaceRoot}/target/x86_64-unknown-linux-gnu/debug/growbe-mainboard",
      "miDebuggerServerAddress": "192.168.50.41:17777",
      "cwd": "${workspaceRoot}",
      "externalConsole": true,
      "linux": {
        "MIMode": "gdb"
      }
    }

	]
}
```

`.vscode/task.json`
```json
{
	"options": {
		"env": {
			"DOCKER_HOST": "127.0.0.1:4243"
		}
	},
    "tasks": [
        {
            "label": "rust: remote ARM debug setup",
            "type": "shell",
            "command": "${workspaceFolder}/scripts/remote_debug.sh",
            "args": [ "${workspaceFolder}", "192.168.50.41", "17777" ],
            "group": "none"
        },
    ]
}
```