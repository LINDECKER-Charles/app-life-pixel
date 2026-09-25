//! The MCP tools and resource through an in-memory transport: an `rmcp` client talks to
//! `LifePixelMcp` over `tokio::io::duplex`, on `InMemoryLibraryStore`, with a policy double and a
//! delivery double in place of the transport's.

mod mcp;
