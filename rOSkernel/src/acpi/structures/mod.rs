pub mod bgrt;
pub mod fadt;
pub mod madt;
pub mod srat;

#[repr(C)]
struct GenericAddressStructure {
    AddressSpace: u8,
    BitWidth: u8,
    BitOffset: u8,
    AccessSize: u8,
    Address: u64,
}
