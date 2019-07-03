## Paths

This needs write access to the following existing directories:
 * `/run/daemon1/public`
 * `/run/daemon2/public`

## Quickstart

Run with:

```
cargo run
```

It will create a metrics file (every 5 seconds) and a unix-domain socket. They can be inspected with:

```
cat /run/daemon1/public/metrics.promfile

socat - UNIX-CONNECT:/run/daemon2/public/metrics.promsock
```
