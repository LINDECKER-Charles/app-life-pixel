# Self-hosting Life Pixel

This folder runs your own Life Pixel server: the `app` image — the server and the web app — and
Postgres, with the documents on a local volume. A self-hosted server never calls Life Pixel's
infrastructure: it sends no telemetry, and mail goes through your own SMTP server.

## Setup

You need Docker with Compose, a domain name pointing to the machine, and an SMTP account: the
server emails address verifications and password resets.

```shell
cp .env.example .env
openssl rand -hex 32   # once per secret of .env
docker compose up -d
docker compose ps      # both services report `healthy`
```

Fill in every `<…>` of `.env` first. The server runs its migrations when it starts, then serves
on port 8080, published on `127.0.0.1` by default (`LP_PUBLISH_ADDRESS`). Two volumes hold the
data: `pgdata`, the database, and `documents`, the animations and exports
(`LP_STORAGE_URL=file:///var/lib/life-pixel`).

To upgrade, change the image's tag in `compose.yaml` to the new version, then:

```shell
docker compose pull
docker compose up -d
```

## HTTPS

Put a reverse proxy in front of port 8080 for HTTPS: sessions and the links of emails need it.
With [Caddy](https://caddyserver.com/) on the same machine, which obtains the certificate:

```text
life-pixel.example.org {
	reverse_proxy 127.0.0.1:8080
}
```

Set `LP_PUBLIC_URL` to `https://life-pixel.example.org`, and `LP_TRUSTED_PROXIES` to the
proxy's address — `127.0.0.1/32` here, or the Docker network's subnet when the proxy runs in a
container — so that rate limits see the visitors' addresses rather than the proxy's.

## Mail

`LP_SMTP_URL` is `smtps://user:password@host:465` for implicit TLS or
`smtp://user:password@host:587` for STARTTLS; percent-encode special characters of the password.
`LP_MAIL_FROM` is the sender, which your provider must allow for the account.

## The admin console (optional)

The console is a second image, `ghcr.io/lindecker-charles/life-pixel/admin`, with its own
database: it reaches the server only through its internal admin API, on port 9091 of the
Compose network, never published. To run it:

1. create a `life_pixel_admin` role and database in Postgres;
2. add an `admin` service on the same version, with the `LPA_` variables of the repository's
   `.env.prod.example` — `LPA_SERVER_ADMIN_API_URL=http://server:9091/internal/admin/v1`, and
   `LPA_SERVER_ADMIN_API_SECRET` equal to `LP_ADMIN_API_SECRET` —, its port 8080 published
   behind the reverse proxy on a domain of its own;
3. create the first admin from the machine:
   `docker compose exec admin life-pixel-admin-server create-admin you@example.org`.

## Backups

Back up both volumes. The database, as a dump to restore with `pg_restore`:

```shell
docker compose exec -T postgres pg_dump -U life_pixel --format=custom life_pixel > life-pixel.dump
```

The documents, as an archive of the `documents` volume, taken while the server is stopped or
right after the dump:

```shell
docker run --rm -v "$(basename "$PWD")_documents:/data:ro" -v "$PWD:/backup" alpine \
  tar -czf /backup/documents.tar.gz -C /data .
```

Keep the copies away from the machine, encrypted, and rehearse a restore before you need one.
