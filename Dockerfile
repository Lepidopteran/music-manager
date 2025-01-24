# Use a base image
FROM debian:stable-slim

# Build arguments
ARG MODE="release"

# Set environment variables
ENV MUUSIK_CONFIG_DIR="/config"
ENV MUUSIK_DATA_DIR="/config/data"
ENV MUUSIK_CACHE_DIR="/cache"
ENV LISTEN_ON_ALL_INTERFACES="true"
ENV PORT="5678" 

# Install dependencies
RUN apt-get update && apt-get install -y sqlite3;

# Copy application code
COPY ./target/release/ /app

# Set permissions
RUN chmod +x /app/muusik

# Set working directory
WORKDIR /app

# Expose application port
EXPOSE $PORT

# Set volumes
VOLUME ["/config", "/cache"]

ENTRYPOINT ["/app/muusik"]
