FROM public.ecr.aws/p6e8q1z1/ink-dev:1.7.0
ARG UID
ARG GID

RUN apt update \
  && apt install -y make
RUN chown -R $UID:$GID /usr/local/cargo
