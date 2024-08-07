# Term-squire
Term-squire is a tool designed for importing .mtf files, which are exported by the TermStar Term Management software.

## Usage:
To run Term-squire, download the appropriate binary for your operating system from the repository's 'releases' section.

`term-squire [OPTIONS]`
`term-squire -p 1234`

Access Term-squire via your web browser at http://ip:port/terms. For local usage, this would be http://localhost:1234/terms.

For a quick start, you can import example.mtf.

### Options:
**-p --port**    
Set the port number for accessing Term-squire. <em>Default: 1234</em>

**-h --help**       
Display help information.

**-v --version**    
Display version information.

# Rust Installation and Cargo Build Instructions
Term-squire is developed in Rust. To build it yourself, you need Rust and Cargo installed on your system.

##  Installing Rust and Cargo
To install Rust and Cargo, you can use the official Rust installation script, `rustup`.

Run the following command in a terminal to install Rust:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

After installation, configure your current shell to use Rust.
`source $HOME/.cargo/env`

Verify the installation by checking the version of Rust and Cargo.
`rustc --version`
`cargo --version`

## Building Term-squire with Cargo
Navigate to the directory containing the Term-squire source code.

Run the following command to build the project:
`cargo build --release`

After the build process completes, the compiled binary will be located in the target/release directory. You can run Term-squire using this binary.

`./term-squire [OPTIONS]`

# API
## Search 
- Endpoint:
```
http://ip:port/search
```
- Example:
```
curl -X GET "http://localhost:1234/search?term=&language="
```

## Insert term
- Endpoint:
```
http://ip:port/insert_term
```
- Example:
```
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

## Add term to term set
- Endpoint:
```
http://ip:port/add_term_set
```
- Example:
```
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

## Update term
- Endpoint:
```
http://ip:port/update_term
```
- Example:
```
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

## Delete
- Endpoint:
```
http://ip:port/delete_term
```
- Example:
```
curl -X DELETE "http://localhost:1234/delete_term?term_id=1"
```


# Q & A:
## What is .mtf? 
Message Text Format (.mtf) is an internationally accepted standard for data interoperability.

## Do you support .mtf?
Yes, Term-squire supports importing .mtf files.

## What database do you use?
Term-squire uses an SQLite database to store terms.