# Stage 1: Build the project using a Rust image
FROM rust:latest as build-stage

# Install wasm-pack
RUN cargo install wasm-pack

# Install Node.js (Example uses a Debian-based Rust image)
RUN curl -fsSL https://deb.nodesource.com/setup_21.x | bash - && \
apt-get install -y nodejs

# Set the working directory in the container
WORKDIR /app

# Copy the project files to the container
COPY . .

# Build the project
RUN wasm-pack build --release --target web

# Move to the assets directory, install Tailwind CSS CLI if needed, and build the CSS
RUN cd assets && npm install tailwindcss && npx tailwindcss -i style.css -o ../pkg/style.css

# Stage 2: Serve the project using nginx
FROM nginx:alpine as serve-stage

# Remove the default nginx static assets
RUN rm -rf /usr/share/nginx/html/*

# Copy the built static assets from the 'pkg' directory in the build stage
COPY --from=build-stage /app/pkg /usr/share/nginx/html

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
