# Task Manager Cot

A full-stack web application built with Rust and the Cot framework. This application provides a complete task management solution featuring passwordless authentication via OTP (One-Time Password), internationalization, and a modern reactive frontend.

## Features

### Authentication & Authorization
- **Passwordless Login**: Users authenticate using a one-time password (OTP) sent to their email.
- **Session Management**: Secure session handling for authenticated users.
- **Auto-User Creation**: New users are automatically registered upon successful OTP verification.

### Task Management (CRUD)
- **Create**: Add new tasks to your personal list.
- **Read**: View all tasks, filtered by "All", "Pending", or "Completed".
- **Update**: Toggle task status between pending and completed.
- **Delete**: Remove tasks from the list.
- **Data Isolation**: Users can only access and manage their own tasks.

### Architecture & UI
- **Internationalization (i18n)**: Full support for English (`en`) and Brazilian Portuguese (`pt-BR`), automatically handled via headers or path routing.
- **Reactive Frontend**: Built with HTML templates, Tailwind CSS for styling, and Alpine.js for lightweight reactivity.
- **CQRS Pattern**: Command and Query Responsibility Segregation applied for database operations.
- **Admin Panel**: Built-in Cot admin dashboard for data management (`/admin`).
- **API Documentation**: Auto-generated OpenAPI specification and Swagger UI (`/swagger`).

## Tech Stack

- **Backend**: Rust, Cot Framework, Tokio
- **Database**: SQLite (configured via `dev.toml`)
- **Frontend**: Alpine.js, Tailwind CSS
- **Serialization**: Serde, Schemars

## Getting Started

### Prerequisites

- Rust (edition 2021) and Cargo installed.
- SQLite installed on your system.

### Running the Project

1. Clone the repository and navigate to the project root.
2. Run the application using Cargo or bacon:

```bash
cargo run
# or
bacon serve
```

3. Open your browser and navigate to `http://localhost:8000` (or the port specified by Cot).

### Development Configuration

The application uses `config/dev.toml` for local development. By default:
- It uses a local SQLite database (`db.sqlite3`).
- Emails (for OTP) are routed to the console (`type = "console"`). Check your terminal output for the OTP code when logging in.
- Live reload middleware is enabled.

## API Endpoints

### Authentication
- `POST /api/auth/otp` - Send an OTP to the provided email.
- `POST /api/auth/verify_otp` - Verify the OTP and establish a session.
- `GET /api/auth/me` - Retrieve current authenticated user data.
- `POST /api/auth/logout` - Destroy the current session.

### Tasks
- `GET /api/tasks` - List all tasks for the authenticated user.
- `POST /api/tasks/create` - Create a new task.
- `PATCH /api/tasks/{id}/update` - Update a task's status.
- `DELETE /api/tasks/{id}/delete` - Delete a task.

## License

This project is licensed under the MIT License.
