#[derive(Debug, Default)]
pub struct SdCid {
    pub manufacturer_id: u8,
    pub application_id: u16,
    pub product_name: [u8; 5],
    pub product_version: u8,
    pub serial_number: u32,
    pub manufacturing_data: u16,
}

impl SdCid {
    pub fn new() -> Self {
        SdCid {
            manufacturer_id: 0,
            application_id: 0,
            product_name: [0; 5],
            product_version: 0,
            serial_number: 0,
            manufacturing_data: 0,
        }
    }
}
