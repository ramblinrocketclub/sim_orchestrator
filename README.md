# sim_orchestrator

This program is configured via enviornment variables, but does support `.env` files. An example of one with variable information can be found in `.env.example`. The deployed instance uses CockroachDB serverless and so does assume some things, such as the existence of a database provided TLS certificate.

The API uses Basic auth with username `bot` and the configured password.
```
GET https://example.com/task

Response body example:
{
  "id": 712988147287031809,
  "data": {
    "tip_b": 11.10250854,
    "root_b": 15.08125677,
    "span_b": 18.56479707,
    "sweep_b": 14.17451454,
    "body_length_b": 116.3380282,
    "tip_s": 11.17326444,
    "root_s": 5.256354696,
    "span_s": 27.75656305,
    "sweep_s": 15.58104842,
    "body_length_s": 8.602383532,
    "body_diameter_bs": 146.4530378,
    "mach_number": 7.438953246,
    "power_on_bs": null,
    "power_off_bs": null,
    "power_on_s": null,
    "power_off_s": null
  }
}

POST https://example.com/task

Request body example:
{
  "id": 712964267027136513,
  "data": {
    "power_on_bs": 1.0,
    "power_off_bs": 2.0,
    "power_on_s": 3.0,
    "power_off_s": 4.0
  }
}
```

```
docker build -t ghcr.io/ramblinrocketclub/sim_orchestrator:latest .
```