FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build
EXPOSE 8000
CMD ["cargo", "shuttle", "run"]