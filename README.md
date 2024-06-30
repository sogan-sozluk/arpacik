# arpacik

Backend service for `soğan sözlük` project.

## Setting up the project

Clone the project and navigate to the project directory.

```bash
$ git clone https://github.com/sogan-sozluk/arpacik
$ cd arpacik
```

Create a environment file and fill the required fields.

```bash
$ cp example.env .env
```

```bash
HOST=127.0.0.1 # Host address
PORT=8099 # Port number
DATABASE_URL='postgres://arpacik:GRoButChN43Wrzt5IXs6hBzLGtFKnRxz@localhost:5499/sogan' # Database URL
DATABASE_SCHEMA='arpacik' # Database schema
JWT_SECRET='VcHCJhYoXGGL7awzIL6woA==' # JWT secret
AUTH_FROM='authorization' # Authorization header. 'cookie' or 'authorization'
```

Create development database and run the migrations.

```bash
$ docker compose up -d
$ cd migration
$ cargo run
```

Run the project.

```bash
# Navigate to the project directory and run the project.
$ cargo run
```

## API Documentation

The API documentation is available as a Postman collection [here](arpacik.postman_collection.json). You can use `cookie` header for authentication.
