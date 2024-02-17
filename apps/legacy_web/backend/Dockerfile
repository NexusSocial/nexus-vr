FROM debian:latest

# Path to the executable to run
ARG BIN_PATH

# Install OpenSSL - required for Axum applications
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Create a new user and group to run the application
RUN groupadd -r app && useradd -r -g app app

RUN mkdir -p /opt/app && chown app:app /opt/app
# Set the working directory
WORKDIR /opt/app


# Switch from root to the "app" user
USER app
COPY --chown=app:app ${BIN_PATH:?missing} /opt/app/backend

# Expose the port the server listens on
EXPOSE 8080 

ENV XDG_DATA_HOME="/opt/app/"
ENV RUST_BACKTRACE=1

# Run the application
CMD "/opt/app/backend"
