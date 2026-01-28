# SSH Operations Management System

An SSH remote command execution and scheduled task management system based on Rust + Tokio.

## Features

### 📦 Resource Management
- **Server Management**: Add, edit, and delete remote servers
- **Group Management**: Organize servers by environment/purpose
- **Cronjob Management**: Configure scheduled command tasks
- **Execution Logs**: Record all command execution history

### ⚡ Command Execution
- **Single Execution**: Execute SSH commands on a single server
- **Batch Execution**: Execute the same command on multiple servers concurrently
- **Async Concurrency**: High-performance concurrent execution based on Tokio

### ⏰ Scheduled Tasks
- Configure frequently-used commands as scheduled tasks
- Support Cron expression scheduling
- Automatically record execution results and logs
