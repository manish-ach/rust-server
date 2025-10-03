### RUST SERVER

currently in this demo we are trying to serve a todo web-app using sqlite db, JWT authorization and bcrypt encrypted data.

The rust server creates a connection with the database, opens port at 8001 in localhost ip.
The CRUD requests and response are handled by axum routers.

### Running
To run it:
- clone the repo
- `cargo run` at the repo
- go to `http://127.0.0.1:8001` to view the site

### Structure

<img width="2188" height="929" alt="image" src="https://github.com/user-attachments/assets/96c3d130-4832-4b44-a9d8-0d492e61fe6c" />
