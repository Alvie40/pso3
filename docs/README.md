# Project Documentation

## Overview
This project is a web application built using the Rust programming language. It leverages the `poem` framework for routing and HTTP handling, and `sqlx` for database interactions. The application includes authentication, routing, and template rendering functionalities.

---

## Modules

### 1. `main`
- **Purpose**: Entry point of the application. Initializes the server, connects to the database, and sets up routes.
- **Key Functions**:
  - `main`: Configures the server and starts listening on `127.0.0.1:3000`.

### 2. `auth`
- **Purpose**: Handles authentication and token management.
- **Submodules**:
  - `handler`: Contains handlers for login and user authentication.
  - `jwt`: Implements JWT token validation and extraction.
  - `dto`: Defines data transfer objects for authentication.
  - `token`: Provides functions to generate and validate JWT tokens.

### 3. `routes`
- **Purpose**: Defines the application's routing logic.
- **Key Functions**:
  - `routes`: Sets up routes for the application, including `/login`, `/me`, and a catch-all for 404 errors.

### 4. `templates`
- **Purpose**: Manages HTML templates for rendering pages.
- **Submodules**:
  - `pages`: Contains handlers for rendering the login page and a 404 page.

### 5. `models`
- **Purpose**: Defines data models used in the application.
- **Submodules**:
  - `mensagens`: Defines the `Mensagem` struct and provides mock data.
  - `users`: Defines user-related data models.

### 6. `state`
- **Purpose**: Manages application state, including database connections.
- **Key Structs**:
  - `AppState`: Represents the shared state of the application.

---

## Key Features

### Authentication
- **Login**: Validates user credentials and generates JWT tokens.
- **Token Validation**: Ensures secure access to protected routes.

### Routing
- **Dynamic Routes**: Handles requests to `/login`, `/me`, and other endpoints.
- **Error Handling**: Provides a custom 404 page for unmatched routes.

### Database Integration
- **PostgreSQL**: Uses `sqlx` for database queries and connection pooling.

### Template Rendering
- **HTML Templates**: Renders pages using static HTML files.

---

## Environment Variables
- `DATABASE_URL`: Connection string for the PostgreSQL database.
- `JWT_SECRET`: Secret key for signing JWT tokens.

---

## How to Run
1. Ensure you have Rust installed.
2. Set up a PostgreSQL database and configure the `.env` file.
3. Run the application using `cargo run`.

---

## Future Improvements
- Add more comprehensive error handling.
- Implement user registration and password recovery features.
- Enhance the UI with additional templates.