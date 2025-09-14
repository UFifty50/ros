#[repr(C)]
pub struct RSDP {
    Signature: [u8; 8],
    Checksum: u8,
    OEMID: [u8; 6],
    Revision: u8,
    RsdtAddress: u32,
}

#[repr(C, packed)]
pub struct XSDP {
    Signature: [u8; 8],
    Checksum: u8,
    OEMID: [u8; 6],
    Revision: u8,
    RsdtAddress: u32,
    Length: u32,
    XsdtAddress: u64,
    ExtendedChecksum: u8,
    Reserved: [u8; 3],
}

impl RSDP {
    pub fn new(address: u64) -> &'static mut Self {
        unsafe { &mut *(address as *mut Self) }
    }

    pub fn get_xsdt(&self) -> Option<&'static XSDP> {
        if self.Revision == 2 {
            let xsdt_address = self.RsdtAddress as usize;
            Some(unsafe { &*(xsdt_address as *const XSDP) })
        } else {
            None
        }
    }
}
