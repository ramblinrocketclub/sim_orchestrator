# sim_orchestrator

This program is configured via enviornment variables, but does support `.env` files. An example of one with variable information can be found in `.env.example`. The deployed instance uses CockroachDB serverless and so does assume some things, such as the existence of a database provided TLS certificate.



```
docker build -t ghcr.io/ramblinrocketclub/sim_orchestrator:latest .
```