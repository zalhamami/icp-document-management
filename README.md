
# Decentralized Document Management System

This is a **Decentralized Document Management System** built on the **Internet Computer Protocol (ICP)** using **Rust**. The system offers a decentralized and tamper-proof way of storing, updating, and managing documents with version history, access control, and features like soft deletion, restoration, and multi-document upload. It is designed for environments where data integrity, security, and immutability are crucial.

## Key Features

### 1. **Document Versioning with Metadata**
   Tracks every modification made to a document and stores each update as a new version, along with metadata such as who made the changes and what was altered. This provides an immutable audit trail, ensuring document integrity and version control, which is essential for environments that require tracking and proof of document authenticity (e.g., legal and scientific records).

### 2. **Decentralized Storage**
   Documents are stored using ICP’s decentralized stable memory, ensuring there is no central authority controlling the data and making it resilient to censorship or tampering. This decentralization provides a secure, trustless environment where the integrity of the documents is guaranteed, ideal for use cases requiring data permanence and security.

### 3. **Soft Delete and Restore**
   Documents can be soft-deleted, which means they are marked as deleted but can be restored later, preventing accidental or unauthorized permanent deletion. This ensures users have a safety net for data recovery, offering flexibility and security by allowing the restoration of crucial documents if needed.

### 4. **Multi-Document Upload**
   Allows users to upload multiple documents in a single operation, reducing the overhead of individually adding documents. This feature improves efficiency, especially when handling large sets of documents, making it suitable for tasks like bulk uploads during migrations or submissions.

### 5. **Search Functionality**
   Provides users with the ability to search documents by title or description, making it easier to retrieve specific documents from the system. This improves the overall user experience and reduces retrieval time, especially when managing a large number of documents.

### 6. **Tamper-Proof and Immutable Audit Logs**
   Every document modification is logged with a timestamp, and previous versions cannot be altered or deleted. This ensures verifiable proof of document history and integrity, which is particularly useful for compliance in legal, academic, or business environments where document authenticity and accountability are paramount.

## How It Works

1. **Add Documents**: Users can upload documents with metadata (title, description, file URL). Each document is stored with a unique ID and an initial version.
   
2. **Update Documents**: Each update to a document creates a new version, and all versions are stored along with metadata such as the timestamp and a summary of changes.

3. **Soft Delete and Restore**: Documents can be soft-deleted, meaning they are marked as deleted but not removed from storage. They can be restored at any time if needed.

4. **Search**: Documents can be searched by title or description, making it easy to retrieve specific documents.

5. **Split Large Fields**: For larger fields (e.g., descriptions or file URLs), the system uses a separate storage mechanism to handle fields that exceed the size limits of ICP’s stable memory.

## Benefits

- **Decentralization**: Operates in a trustless environment, without central control, ensuring data is secure and censorship-resistant.
- **Immutability**: Document changes are logged as new versions, and older versions remain immutable, ensuring a tamper-proof audit trail.
- **Scalability**: By using chunking for large fields, the system can handle large volumes of data while optimizing storage use.
- **Data Integrity**: Full version control and logging ensure that documents maintain integrity over time, with a verifiable history of modifications.
- **Accidental Recovery**: Soft delete allows users to recover documents that were accidentally deleted, adding an additional layer of security for critical data.

## Potential Use Cases

- **Legal Document Storage**: Ensuring legal documents are stored with complete version history and tamper-proof logs.
- **Academic Research**: Tracking the evolution of scientific research papers, along with versions and metadata.
- **Corporate Compliance**: Ensuring regulatory documents are stored immutably and securely with full audit trails.

## Getting Started

To get started, deploy this code on an **Internet Computer Canister**. Use **Candid UI** or a custom front-end to interact with the system.

### Install Dependencies

Make sure you have the following dependencies in your project:

```toml
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

### Compile and Deploy

1. Compile the canister using Rust and ICP’s toolchain.
2. Deploy the canister to your ICP node.
3. Interact with it using the provided Candid interface or build a custom front-end.

---

This system provides a powerful decentralized solution for document management with strong guarantees of data integrity, security, and version control. It is suitable for various industries that require reliable, tamper-proof record keeping.
