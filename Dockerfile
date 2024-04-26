FROM rust:alpine as build

WORKDIR /root

ENV SQLX_OFFLINE=true

COPY . .

RUN apk add \
  musl-dev \
  pkgconfig \
  libressl-dev
RUN cargo build --release

FROM alpine

COPY --from=build /root/target/release/tcg-scraper /bin/tcg-scraper

ENTRYPOINT ["tcg-scraper"]
