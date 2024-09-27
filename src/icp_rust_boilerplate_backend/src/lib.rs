#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Metadata for document updates
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct DocumentMetadata {
    updated_by: String,
    change_summary: String,
}

// Document struct stored in stable storage
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Document {
    id: u64,
    title: String,
    description: String,
    file_url: String,
    version: u64,
    created_at: u64,
    updated_at: Option<u64>,
    is_deleted: bool,
    history: Vec<DocumentVersion>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct DocumentVersion {
    version: u64,
    title: String,
    description: String,
    file_url: String,
    metadata: DocumentMetadata,
    updated_at: u64,
}

// Document payload for creating or updating a document
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct DocumentPayload {
    title: String,
    description: String,
    file_url: String,
    metadata: DocumentMetadata,
}

// Storable trait for Document
impl Storable for Document {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

// BoundedStorable trait for Document
impl BoundedStorable for Document {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Thread-local storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, Document, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

// Payload validation function
fn validate_document_payload(payload: &DocumentPayload) -> Result<(), Error> {
    if payload.title.trim().is_empty() {
        return Err(Error::InvalidInput { msg: "Title cannot be empty".to_string() });
    }
    if payload.description.trim().is_empty() {
        return Err(Error::InvalidInput { msg: "Description cannot be empty".to_string() });
    }
    if payload.file_url.trim().is_empty() {
        return Err(Error::InvalidInput { msg: "File URL cannot be empty".to_string() });
    }
    if payload.metadata.change_summary.trim().is_empty() {
        return Err(Error::InvalidInput { msg: "Change summary cannot be empty".to_string() });
    }
    Ok(())
}

// Function to add multiple documents at once
#[ic_cdk::update]
fn add_documents(documents: Vec<DocumentPayload>) -> Result<Vec<Document>, Error> {
    let mut added_documents = Vec::new();

    for payload in documents {
        let document = add_single_document(payload.clone())?;
        added_documents.push(document);
    }

    Ok(added_documents)
}

fn add_single_document(payload: DocumentPayload) -> Result<Document, Error> {
    // Validate the payload
    validate_document_payload(&payload)?;

    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
    }).expect("cannot increment id counter");

    let document = Document {
        id,
        title: payload.title.clone(),
        description: payload.description.clone(),
        file_url: payload.file_url.clone(),
        version: 1,
        created_at: time(),
        updated_at: None,
        is_deleted: false,
        history: vec![DocumentVersion {
            version: 1,
            title: payload.title.clone(),
            description: payload.description.clone(),
            file_url: payload.file_url.clone(),
            metadata: payload.metadata.clone(),
            updated_at: time(),
        }],
    };

    do_insert_document(&document);
    Ok(document)
}

fn do_insert_document(document: &Document) {
    STORAGE.with(|service| service.borrow_mut().insert(document.id, document.clone()));
}

// Update a document and track version history with metadata
#[ic_cdk::update]
fn update_document(id: u64, payload: DocumentPayload) -> Result<Document, Error> {
    // Validate the payload
    validate_document_payload(&payload)?;

    STORAGE.with(|service| {
        match service.borrow().get(&id) {
            Some(mut document) => {
                if document.is_deleted {
                    return Err(Error::DocumentDeleted);
                }

                let new_version = document.version + 1;
                let doc_version = DocumentVersion {
                    version: new_version,
                    title: payload.title.clone(),
                    description: payload.description.clone(),
                    file_url: payload.file_url.clone(),
                    metadata: payload.metadata.clone(),
                    updated_at: time(),
                };
                document.history.push(doc_version);

                document.title = payload.title;
                document.description = payload.description;
                document.file_url = payload.file_url;
                document.version = new_version;
                document.updated_at = Some(time());

                do_insert_document(&document);
                Ok(document)
            }
            None => Err(Error::NotFound { msg: format!("Document with id {} not found", id) }),
        }
    })
}

// Soft delete document, can be restored later
#[ic_cdk::update]
fn soft_delete_document(id: u64) -> Result<Document, Error> {
    STORAGE.with(|service| {
        let mut storage = service.borrow_mut();
        
        if let Some(mut document) = storage.remove(&id) {
            if document.is_deleted {
                // If already deleted, return an error
                storage.insert(id, document); // Reinserting the document back if no update is made
                return Err(Error::AlreadyDeleted);
            }
            
            // Mark the document as deleted and reinsert it
            document.is_deleted = true;
            storage.insert(id, document.clone());
            Ok(document)
        } else {
            // Document not found
            Err(Error::NotFound { msg: format!("Document with id {} not found", id) })
        }
    })
}

// Restore a soft-deleted document
#[ic_cdk::update]
fn restore_document(id: u64) -> Result<Document, Error> {
    STORAGE.with(|service| {
        let mut storage = service.borrow_mut();
        
        if let Some(mut document) = storage.remove(&id) {
            if !document.is_deleted {
                // If not deleted, return an error
                storage.insert(id, document); // Reinserting the document back if no update is made
                return Err(Error::NotDeleted);
            }
            
            // Mark the document as restored and reinsert it
            document.is_deleted = false;
            storage.insert(id, document.clone());
            Ok(document)
        } else {
            // Document not found
            Err(Error::NotFound { msg: format!("Document with id {} not found", id) })
        }
    })
}

// Search for documents by title or description
#[ic_cdk::query]
fn search_documents(query: String) -> Vec<Document> {
    STORAGE.with(|service| {
        let all_docs: Vec<Document> = service.borrow().iter().map(|(_, doc)| doc.clone()).collect();
        all_docs.into_iter().filter(|doc| {
            doc.title.to_lowercase().contains(&query.to_lowercase()) ||
            doc.description.to_lowercase().contains(&query.to_lowercase())
        }).collect()
    })
}

// Retrieve a document by ID
#[ic_cdk::query]
fn get_document(id: u64) -> Result<Document, Error> {
    STORAGE.with(|s| match s.borrow().get(&id) {
        Some(document) if !document.is_deleted => Ok(document.clone()),
        Some(_) => Err(Error::DocumentDeleted),
        None => Err(Error::NotFound { msg: format!("Document with id {} not found", id) }),
    })
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    DocumentDeleted,
    AlreadyDeleted,
    NotDeleted,
    InvalidInput { msg: String },
}

// Export candid interface
ic_cdk::export_candid!();
