# beartamer

Endpoint: `/devices`
Supported functions:

- `GET /devices` - get all known devices as an array.

Endpoint: `/secrets/:domain?device_id=:device_id`
It is strongly recommended to always pass `device_id` in all requests.
Supported functions:

- `GET /secrets/:domain` - get by domain name,
- `GET /secrets` - get list of all secrets,
- `PUT /secrets/:domain` or `POST /secrets/:domain` - insert/update secret,
- `DELETE /secrets/:domain` - delete secret by domain name.

Device id is optional, if passed, the querying device will be registered in the system.
If device id is missing, IP is stored instead.

Data structure:

```
{
  "domain: "string",
  "username": "string",
  "password": "string",
  "type": "password"
}
```

```
{
  "domain: "string",
  "number": "1111222233334444",
  "cvc": "123",
  "fullname": "Ivan Ivanoff",
  "year": 2040,
  "month": 12,
  "type": "creditcard"
}
```

Run `cargo build -- 0.0.0.0:9000` to build.
Run `cargo run -- 0.0.0.0:9000` to build and run.

Install `openssl` library on Linux before build.
