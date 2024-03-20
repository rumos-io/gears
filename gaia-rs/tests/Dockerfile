FROM rust:slim AS runtime

COPY . /gears

ENV BECH_32_MAIN_PREFIX="cosmos"

RUN rm -drf /gears/target 

RUN apt-get update && apt-get upgrade -y
RUN apt-get install clang llvm git -y