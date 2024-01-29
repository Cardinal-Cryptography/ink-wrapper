FROM public.ecr.aws/p6e8q1z1/ink-dev:2.0.1
ARG UID
ARG GID

RUN apt update \
  && apt install -y make
RUN chown -R $UID:$GID /usr/local/cargo
