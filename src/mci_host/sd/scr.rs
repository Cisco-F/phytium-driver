use bitflags::bitflags;

#[derive(Debug, Default)]
pub struct SdScr {
    // SCR Structure [63:60]
    pub scr_structure: u8,
    // SD memory card specification version [59:56]
    pub sd_specification: u8,
    // SCR flags in SdScrFlag
    pub flags: u16,
    // Security specification supported [54:52]
    pub sd_security: u8,
    // Data bus widths supported [51:48]
    pub sd_bus_widths: u8,
    // Extended security support [46:43]
    pub extended_security: u8,
    // Command support bits [33:32] 33-support CMD23, 32-support cmd20
    pub command_support: u8,
    // Reserved for manufacturer usage [31:0]
    pub reserved_for_manufacturer: u32,
}

impl SdScr {
    pub fn new() -> Self {
        SdScr {
            scr_structure: 0,
            sd_specification: 0,
            flags: 0,
            sd_security: 0,
            sd_bus_widths: 0,
            extended_security: 0,
            command_support: 0,
            reserved_for_manufacturer: 0,
        }
    }
}

bitflags! {
    pub struct ScrFlags: u16 {
        const DATA_STATUS_AFTER_ERASE = 1 << 0; /* Data status after erases [55:55] */
        const SD_SPECIFICATION3 = 1 << 1; /* SD specification 3.00 or higher [47:47] */
    }
}
