# ink-wrapper

## CI

You should be able to run the CI processes locally with https://github.com/nektos/act:

```bash
$ act -v
```

Note that if you have docker installed in rootless mode you might need to provide the socket to `act` manually,
something like:

```bash
$ act -v --container-daemon-socket $DOCKER_HOST
```

Alternatively, see `.github/workflows/ci.yml` for the dockerized steps taken and customize for yourself (such as
starting permanent containers to run the steps in) to speed up the feedback loop locally.
