# beartamer

Endpoint: `/:domain`
Supported functions:

- `GET /:domain` - get by domain name,
- `GET /` - get list of all secrets,
- `PUT /:domain` or `POST /:domain` - insert/update secret,
- `DELETE /:domain` - delete secret by domain name.

Data structure:

```
{
  "domain: "string",
  "username": "string",
  "password": "string",
  "type": "password"
}
```

Run `cargo build -- 0.0.0.0:9000` to build.
Run `cargo run -- 0.0.0.0:9000` to build and run.

Install `openssl` library on Linux before build.
