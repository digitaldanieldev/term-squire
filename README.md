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
