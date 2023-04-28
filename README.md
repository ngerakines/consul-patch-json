# consul-patch-json

A small tool used to patch JSON values in consul.

Usage:

    $ consul-patch-json apps/foo/config version='"1.0.0"'

# Installation

    $ cargo install consul-patch-json

# Consul Configuration

The following environment variables are used to configure consul interaction:

* `CONSUL_HTTP_ADDR`
* `CONSUL_CACERT`
* `CONSUL_CAPATH`
* `CONSUL_CLIENT_CERT`
* `CONSUL_CLIENT_KEY`
* `CONSUL_HTTP_TOKEN`
* `CONSUL_HTTP_SSL_VERIFY`

# Examples

# Adding attributes

    $ consul kv put apps/foo/config '{"version": "1.0.0"}'
    $ consul-patch-json apps/foo/config description='"My app"'
    $ consul kv get apps/foo/config
    {"description":"My app","version":"1.0.0"}

# Replacing attributes

    $ consul kv put apps/foo/config '{"version": "1.0.0"}'
    $ consul-patch-json apps/foo/config version='"1.0.1"'
    $ consul kv get apps/foo/config
    {"version":"1.0.1"}

# Complex attributes

    $ consul kv put apps/foo/config '{"version": "1.0.0"}'
    $ consul-patch-json apps/foo/config features='["metrics"]'
    $ consul kv get apps/foo/config
    {"features":["metrics"],"version":"1.0.0"}

# Reading standard input for single attributes

    $ consul kv put apps/foo/config '{"version": "1.0.0"}'
    $ echo '["coffee"]' | consul-patch-json apps/foo/config features=--
    $ consul kv get apps/foo/config
    {"features":["coffee"],"version":"1.0.0"}

# JSON Merge Patch (RFC 7396)

    $ consul kv put apps/foo/config '{"version": "1.0.0"}'
    $ jo features=$(jo -a coffee metrics) | consul-patch-json apps/foo/config --
    $ consul kv get apps/foo/config
    {"features":["coffee","metrics"],"version":"1.0.0"}

See also: https://datatracker.ietf.org/doc/html/rfc7396

# JSON Patch (RFC 6902)

    $ consul kv put apps/foo/config '{"version": "1.0.0","features":["coffee"]}'
    $ cat > patch.json <<EOF
    [
        {"op": "test", "path": "/version", "value": "1.0.0"},
        {"op": "add","path": "/features/0", "value": "metrics"}
    ]
    EOF
    $ cat patch.json | consul-patch-json apps/foo/config --json-patch --
    $ consul kv get apps/foo/config
    {"features":["coffee","metrics"],"version":"1.0.0"}

See also: https://datatracker.ietf.org/doc/html/rfc6902

# TODO

- [ ] Support transactions
