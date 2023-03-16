# ink-wrapper

## CI

Use the commands provided in the `Makefile` to replicate the build process run on CI. The most hassle-free is to just
run everything in docker:

```bash
make all-dockerized
```

If you have the tooling installed on your host and start a node yourself, you can also run the build on your host:

```bash
make all
```

In case there are any runaway containers from `all-dockerized` you can kill them:

```bash
make kill
```
