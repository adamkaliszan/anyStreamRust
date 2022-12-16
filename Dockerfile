FROM rust AS anyStreamRust
LABEL "maintainer"="Adam Kaliszan<adam.kaliszan@gmail.com>"
LABEL "about"="Any stream simulator container image"

WORKDIR /AnyStream
COPY . .

RUN cargo build -r
RUN ln -s ./target/release/any_stream /usr/local/bin

