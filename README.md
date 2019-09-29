# beartamer

## Installation and setup

Install `openssl` library with headers on Linux before build.

Make sure you have a MongoDB instance available.
Configure connection to MongoDB in `config.json` file (see later).
To setup MongoDB instance with Docker, use these commands:

```
docker volume create --name=mongodata
docker run --name mongodb -v mongodata:/data/db -d -p 27017:27017 mongo
```

Run `cargo build -- 0.0.0.0:9000` to build.
Run `cargo run -- 0.0.0.0:9000` to build and run.

If you omit the `0.0.0.0:9000`, the server will run on `127.0.0.1:9000`.

## Endpoints

### Endpoint: `/devices`

Supported methods:

- `GET /devices` - get all known devices as an array.

Data structure:

```json
[
  {
    "device_id": "string",
    "ip_addrs": [
      "1.2.3.4",
      "5.6.7.8"
    ]
  }
]
```

### Endpoint: `/secrets/:domain?device_id=:device_id`

Supported methods:

- `GET /secrets/:domain` - get by domain name,
- `GET /secrets` - get list of all secrets,
- `PUT /secrets/:domain` or `POST /secrets/:domain` - insert/update secret,
- `DELETE /secrets/:domain` - delete secret by domain name.

Device id is optional. If device id is missing, `"unknown"` string is used instead.
It is strongly recommended to always pass `device_id` in all requests.

Data structure:

```json
{
  "domain": "string",
  "username": "string",
  "password": "string",
  "type": "password"
}
```

```json
{
  "domain": "string",
  "number": "1111222233334444",
  "cvc": "123",
  "fullname": "Ivan Ivanoff",
  "year": 2040,
  "month": 12,
  "type": "creditcard"
}
```

## Config

Use config file `config.json` to configure MongoDB connection:

```json
{
  "host": "localhost",
  "port": 27017,
  "dbname": "keypass",
  "pool_size": 16,
  "username": "root",
  "password": "passw0rd"
}
```

The `"username"` and `"password"` fields are optional.
