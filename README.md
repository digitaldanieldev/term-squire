# Term-squire
Term-squire is a term management tool designed for translation workflows. It helps manage and organize terminology data efficiently and supports importing `.mtf` files exported by the TermStar Term Management software. 

---

## Usage
To run Term-squire, download the appropriate binary for your operating system from the repository's **Releases** section.

### Command-Line Usage
```bash
term-squire [OPTIONS]
```

#### Example: Run Term-squire Locally
```bash
term-squire -p 1234
```
- `-p`: Specifies the port number for accessing Term-squire (default: `1234`).

Access Term-squire via your web browser:
- **Local:** [http://localhost:1234/terms](http://localhost:1234/terms)
- **Remote:** Replace `localhost` with your server's IP.

For a quick start, you can import `example.mtf` for testing.

### Available Options:
- **`-p --port`**  
   Set the port number for accessing Term-squire. *(Default: 1234)*
- **`-d --data_dir`**  
   Set the database location. *(Default: /data/term-squire-data)*
- **`- --log_level`**  
   Set the logging level. *(Default: "info")*
   Available options: ["error, warn, info, debug, trace"]
- **`-h --help`**  
   Display help information about the application.
- **`-v --version`**  
   Display version information.

---

## Rust Installation and Cargo Build Instructions
Term-squire is developed in Rust. To build it yourself, you need Rust and Cargo installed on your system.

### Installing Rust and Cargo
To install Rust and Cargo, you can use the official Rust installation script, `rustup`.

#### Steps:
1. Run the following command in a terminal:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. After installation, configure your current shell to use Rust:
   ```bash
   source $HOME/.cargo/env
   ```
3. Verify the installation by checking the version of Rust and Cargo:
   ```bash
   rustc --version
   cargo --version
   ```

### Building Term-squire with Cargo
1. Navigate to the directory containing the Term-squire source code.
2. Run the following command to build the project:
   ```bash
   cargo build --release
   ```
3. After the build process completes, the compiled binary will be located in the `target/release` directory. Run Term-squire using this binary:
   ```bash
   ./term-squire [OPTIONS]
   ```

---

## API Documentation

### 1. Search Terms
- **Endpoint:**
  ```
  http://ip:port/search
  ```
- **Example:**
  ```bash
  curl -X GET "http://localhost:1234/search?term=&language="
  ```

### 2. Insert Term
- **Endpoint:**
  ```
  http://ip:port/insert_term
  ```
- **Example:**
  ```bash
  curl -X POST http://localhost:1234/insert_term \
       -H "Content-Type: application/json" \
       -d '{
             "term_language_set": {
               "term": "example_term",
               "language": "en",
               "term_type": "noun",
               "creator_id": "user123",
               "updater_id": "user123",
               "subject": "general",
               "source": "source",
               "user": "user123",
               "attributes": "attribute",
               "remark": "remark",
               "url": "http://example.com",
               "context": "context",
               "definition": "definition"
             }
           }'
  ```

### 3. Add Term to Term Set
- **Endpoint:**
  ```
  http://ip:port/add_term_set
  ```
- **Example:**
  ```bash
  curl -X POST http://localhost:1234/add_term_set \
       -H "Content-Type: application/json" \
       -d '{
             "existing_term_set_id": 1,
             "term_language_set": {
               "term": "example_term",
               "language": "en",
               "term_type": "noun",
               "creator_id": "user123",
               "updater_id": "user123",
               "subject": "general",
               "source": "source",
               "user": "user123",
               "attributes": "attribute",
               "remark": "remark",
               "url": "http://example.com",
               "context": "context",
               "definition": "definition"
             }
           }'
  ```

### 4. Update Term
- **Endpoint:**
  ```
  http://ip:port/update_term
  ```
- **Example:**
  ```bash
  curl -X POST http://localhost:1234/update_term \
       -H "Content-Type: application/json" \
       -d '{
             "term_id": 1,
             "term_language_set": {
               "term": "updated_term",
               "language": "es",
               "term_type": "adjective",
               "creator_id": "user789",
               "updater_id": "user789",
               "subject": "updated_subject",
               "source": "updated_source",
               "user": "user789",
               "attributes": "updated_attribute",
               "remark": "updated_remark",
               "url": "http://updatedexample.com",
               "context": "updated_context",
               "definition": "updated_definition"
             }
           }'
  ```

### 5. Delete Term
- **Endpoint:**
  ```
  http://ip:port/delete_term
  ```
- **Example:**
  ```bash
  curl -X DELETE "http://localhost:1234/delete_term?term_id=1"
  ```

### 6. Download Database
- **Endpoint:**
  ```
  http://ip:port/download_db_file
  ```
- **Example:**
  ```bash
  curl -X GET "http://localhost:1234/download_db_file" -o term-squire.db
  ```

### 7. Upload Database
- **Endpoint:**
  ```
  http://ip:port/upload_db_file
  ```
- **Example:**
  ```bash
  curl -X POST "http://localhost:1234/upload_db_file" \
      -F "file=@term-squire.db"
  ```

---

## Q & A

### What is `.mtf`?
Message Text Format (.mtf) is an internationally accepted standard for data interoperability.

### Does Term-squire support `.mtf`?
Yes, Term-squire supports importing `.mtf` files.

### What database does Term-squire use?
Term-squire uses an SQLite database to store terms.

### Can I look inside the database?
Yes, you can download the database from Term-squire and use the [SQLite Database Browser](https://sqlitebrowser.org/dl/) to view its contents.

---

---

## Running Term-squire with Docker

You can also run Term-squire using Docker for easier deployment and environment management.

### Using Docker Compose

Here is a sample `docker-compose.yml` setup that runs both **Term-squire** and an **nginx-auth** reverse proxy with SSL:

```yaml
services:
  term-squire:
    build:
      context: ./term-squire
      dockerfile: Dockerfile
    container_name: term-squire
    ports:
      - "12500:12500"
    networks:
      network:
        ipv4_address: 172.27.0.2
    volumes:
      - ./ssl:/etc/ssl
      - /my/nas/docker/term-squire:/data/term-squire-data
    restart: always

  nginx-auth:
    depends_on:
      - term-squire
    build:
      context: ./nginx-auth
      dockerfile: Dockerfile
    container_name: nginx-auth
    ports:
      - "12443:443"
    networks:
      network:
        ipv4_address: 172.27.0.3
    volumes:
      - ./ssl:/etc/ssl
      - /my/nas/docker/nginx-logs/term-squire:/var/log/nginx
    restart: always

networks:
  network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.27.0.0/24
          gateway: 172.27.0.1
```


## Dockerfile for term-squire

Example term-squire/Dockerfile used in the above configuration:

```
# Use the base image
FROM ubuntu:22.04

# Set the working directory inside the container
WORKDIR /usr/local/bin

# Install dependencies
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*

# Copy the pre-built Term-squire binary from the host
COPY ./term-squire /usr/local/bin/term-squire

# Make sure the binary is executable
RUN chmod +x /usr/local/bin/term-squire

# Expose the application's port
EXPOSE 12500

# Run the application
CMD ["term-squire", "-p", "12500"]
```
