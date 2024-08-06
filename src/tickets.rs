//! This module handles the ticket logic

// Ticket actions
mod claim;
mod close;
mod create;

// Re-exports of the ticket actions
pub use claim::claim as claim_ticket;
pub use close::close as close_ticket;
pub use create::create as create_ticket;

/// The emoji used for tickets
pub const TICKET_EMOJI: &str = "🎫";
