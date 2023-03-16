FROM public.ecr.aws/p6e8q1z1/ink-dev:1.0.0

RUN apt update \
  && apt install -y make
