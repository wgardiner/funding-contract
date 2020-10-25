pub mod contract;
// exclude error module for cosmwasm v0.10.0
//pub mod error;
pub mod msg;
pub mod state;

#[cfg(test)]
mod tests;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points!(contract);
