#[derive(Debug, Clone, Copy)]
pub enum FioPadFunc {
    Func0 = 0b000,
    Func1,
    Func2,
    Func3 = 0b011,
    Func4,
    Func5,
    Func6,
    Func7 = 0b111,

    NumOfFunc,
}

impl From<u32> for FioPadFunc {
    fn from(value: u32) -> Self {
        match value {
            0b000 => FioPadFunc::Func0,
            0b001 => FioPadFunc::Func1,
            0b010 => FioPadFunc::Func2,
            0b011 => FioPadFunc::Func3,
            0b100 => FioPadFunc::Func4,
            0b101 => FioPadFunc::Func5,
            0b110 => FioPadFunc::Func6,
            0b111 => FioPadFunc::Func7,
            _ => panic!("Invalid value for FioPadFunc"),
        }
    }
}

impl Into<u32> for FioPadFunc {
    fn into(self) -> u32 {
        match self {
            FioPadFunc::Func0 => 0b000,
            FioPadFunc::Func1 => 0b001,
            FioPadFunc::Func2 => 0b010,
            FioPadFunc::Func3 => 0b011,
            FioPadFunc::Func4 => 0b100,
            FioPadFunc::Func5 => 0b101,
            FioPadFunc::Func6 => 0b110,
            FioPadFunc::Func7 => 0b111,
            _ => panic!("Invalid value for FioPadFunc"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FioPadDrive {
    Drv0 = 0b0000,
    Drv1,
    Drv2,
    Drv3,
    Drv4,
    Drv5,
    Drv6,
    Drv7,
    Drv8,
    Drv9,
    Drv10,
    Drv11,
    Drv12,
    Drv13,
    Drv14,
    Drv15 = 0b1111,

    NumOfDrive,
}

impl From<u32> for FioPadDrive {
    fn from(value: u32) -> Self {
        match value {
            0b0000 => FioPadDrive::Drv0,
            0b0001 => FioPadDrive::Drv1,
            0b0010 => FioPadDrive::Drv2,
            0b0011 => FioPadDrive::Drv3,
            0b0100 => FioPadDrive::Drv4,
            0b0101 => FioPadDrive::Drv5,
            0b0110 => FioPadDrive::Drv6,
            0b0111 => FioPadDrive::Drv7,
            0b1000 => FioPadDrive::Drv8,
            0b1001 => FioPadDrive::Drv9,
            0b1010 => FioPadDrive::Drv10,
            0b1011 => FioPadDrive::Drv11,
            0b1100 => FioPadDrive::Drv12,
            0b1101 => FioPadDrive::Drv13,
            0b1110 => FioPadDrive::Drv14,
            0b1111 => FioPadDrive::Drv15,
            _ => panic!("Invalid value for FioPadDrive"),
        }
    }
}

impl Into<u32> for FioPadDrive {
    fn into(self) -> u32 {
        match self {
            FioPadDrive::Drv0 => 0b0000,
            FioPadDrive::Drv1 => 0b0001,
            FioPadDrive::Drv2 => 0b0010,
            FioPadDrive::Drv3 => 0b0011,
            FioPadDrive::Drv4 => 0b0100,
            FioPadDrive::Drv5 => 0b0101,
            FioPadDrive::Drv6 => 0b0110,
            FioPadDrive::Drv7 => 0b0111,
            FioPadDrive::Drv8 => 0b1000,
            FioPadDrive::Drv9 => 0b1001,
            FioPadDrive::Drv10 => 0b1010,
            FioPadDrive::Drv11 => 0b1011,
            FioPadDrive::Drv12 => 0b1100,
            FioPadDrive::Drv13 => 0b1101,
            FioPadDrive::Drv14 => 0b1110,
            FioPadDrive::Drv15 => 0b1111,
            _ => panic!("Invalid value for FioPadDrive"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FioPadPull {
    PullNone = 0b00,
    PullDown = 0b01,
    PullUp = 0b10,

    NumOfPull,
}

impl From<u32> for FioPadPull {
    fn from(value: u32) -> Self {
        match value {
            0b00 => FioPadPull::PullNone,
            0b01 => FioPadPull::PullDown,
            0b10 => FioPadPull::PullUp,
            _ => panic!("Invalid value for FioPadPull"),
        }
    }
}

impl Into<u32> for FioPadPull {
    fn into(self) -> u32 {
        match self {
            FioPadPull::PullNone => 0b00,
            FioPadPull::PullDown => 0b01,
            FioPadPull::PullUp => 0b10,
            _ => panic!("Invalid value for FioPadPull"),
        }
    }
}

impl FioPadPull {
    pub fn is_pull_none(&self) -> bool {
        match self {
            FioPadPull::PullNone => true,
            _ => false,
        }
    }

    pub fn is_pull_down(&self) -> bool {
        match self {
            FioPadPull::PullDown => true,
            _ => false,
        }
    }

    pub fn is_pull_up(&self) -> bool {
        match self {
            FioPadPull::PullUp => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FioPadDelayDir {
    OutputDelay = 0, // Delay setting direction to output
    InputDelay,      // Delay setting direction to input

    NumOfDelayDir,
}

#[derive(Debug, PartialEq)]
pub enum FioPadDelayType {
    DelayCoarseTuning = 0, // delay coarse tuning
    DelayFineTuning,       // delay fine tuning

    NumOfDelayType,
}

#[derive(Debug, Clone, Copy)]
pub enum FioPadDelay {
    DelayNone = 0,
    Delay1,
    Delay2,
    Delay3,
    Delay4,
    Delay5,
    Delay6,
    Delay7,

    NumOfDelay,
}

impl From<u32> for FioPadDelay {
    fn from(value: u32) -> Self {
        match value {
            0 => FioPadDelay::DelayNone,
            1 => FioPadDelay::Delay1,
            2 => FioPadDelay::Delay2,
            3 => FioPadDelay::Delay3,
            4 => FioPadDelay::Delay4,
            5 => FioPadDelay::Delay5,
            6 => FioPadDelay::Delay6,
            7 => FioPadDelay::Delay7,
            _ => panic!("Invalid value for FioPadDelay"),
        }
    }
}

impl Into<u32> for FioPadDelay {
    fn into(self) -> u32 {
        match self {
            FioPadDelay::DelayNone => 0,
            FioPadDelay::Delay1 => 1,
            FioPadDelay::Delay2 => 2,
            FioPadDelay::Delay3 => 3,
            FioPadDelay::Delay4 => 4,
            FioPadDelay::Delay5 => 5,
            FioPadDelay::Delay6 => 6,
            FioPadDelay::Delay7 => 7,
            _ => panic!("Invalid value for FioPadDelay"),
        }
    }
}

// register offset of iopad function / pull / driver strength
pub const FIOPAD_AN59_REG0_OFFSET: u32 = 0x0000;
pub const FIOPAD_AW47_REG0_OFFSET: u32 = 0x0004;
pub const FIOPAD_AR55_REG0_OFFSET: u32 = 0x0020;
pub const FIOPAD_AJ55_REG0_OFFSET: u32 = 0x0024;
pub const FIOPAD_AL55_REG0_OFFSET: u32 = 0x0028;
pub const FIOPAD_AL53_REG0_OFFSET: u32 = 0x002C;
pub const FIOPAD_AN51_REG0_OFFSET: u32 = 0x0030;
pub const FIOPAD_AR51_REG0_OFFSET: u32 = 0x0034;
pub const FIOPAD_BA57_REG0_OFFSET: u32 = 0x0038;
pub const FIOPAD_BA59_REG0_OFFSET: u32 = 0x003C;
pub const FIOPAD_AW57_REG0_OFFSET: u32 = 0x0040;
pub const FIOPAD_AW59_REG0_OFFSET: u32 = 0x0044;
pub const FIOPAD_AU55_REG0_OFFSET: u32 = 0x0048;
pub const FIOPAD_AN57_REG0_OFFSET: u32 = 0x004C;
pub const FIOPAD_AL59_REG0_OFFSET: u32 = 0x0050;
pub const FIOPAD_AJ59_REG0_OFFSET: u32 = 0x0054;
pub const FIOPAD_AJ57_REG0_OFFSET: u32 = 0x0058;
pub const FIOPAD_AG59_REG0_OFFSET: u32 = 0x005C;
pub const FIOPAD_AG57_REG0_OFFSET: u32 = 0x0060;
pub const FIOPAD_AE59_REG0_OFFSET: u32 = 0x0064;
pub const FIOPAD_AC59_REG0_OFFSET: u32 = 0x0068;
pub const FIOPAD_AC57_REG0_OFFSET: u32 = 0x006C;
pub const FIOPAD_AR49_REG0_OFFSET: u32 = 0x0070;
pub const FIOPAD_BA55_REG0_OFFSET: u32 = 0x0074;
pub const FIOPAD_BA53_REG0_OFFSET: u32 = 0x0078;
pub const FIOPAD_AR59_REG0_OFFSET: u32 = 0x007C;
pub const FIOPAD_AU59_REG0_OFFSET: u32 = 0x0080;
pub const FIOPAD_AR57_REG0_OFFSET: u32 = 0x0084;
pub const FIOPAD_BA49_REG0_OFFSET: u32 = 0x0088;
pub const FIOPAD_AW55_REG0_OFFSET: u32 = 0x008C;
pub const FIOPAD_A35_REG0_OFFSET: u32 = 0x0090;
pub const FIOPAD_R57_REG0_OFFSET: u32 = 0x0094;
pub const FIOPAD_R59_REG0_OFFSET: u32 = 0x0098;
pub const FIOPAD_U59_REG0_OFFSET: u32 = 0x009C;
pub const FIOPAD_W59_REG0_OFFSET: u32 = 0x00A0;
pub const FIOPAD_U57_REG0_OFFSET: u32 = 0x00A4;
pub const FIOPAD_AA57_REG0_OFFSET: u32 = 0x00A8;
pub const FIOPAD_AA59_REG0_OFFSET: u32 = 0x00AC;
pub const FIOPAD_AW51_REG0_OFFSET: u32 = 0x00B0;
pub const FIOPAD_AU51_REG0_OFFSET: u32 = 0x00B4;
pub const FIOPAD_A39_REG0_OFFSET: u32 = 0x00B8;
pub const FIOPAD_C39_REG0_OFFSET: u32 = 0x00BC;
pub const FIOPAD_C37_REG0_OFFSET: u32 = 0x00C0;
pub const FIOPAD_A37_REG0_OFFSET: u32 = 0x00C4;
pub const FIOPAD_A41_REG0_OFFSET: u32 = 0x00C8;
pub const FIOPAD_A43_REG0_OFFSET: u32 = 0x00CC;
pub const FIOPAD_A45_REG0_OFFSET: u32 = 0x00D0;
pub const FIOPAD_C45_REG0_OFFSET: u32 = 0x00D4;
pub const FIOPAD_A47_REG0_OFFSET: u32 = 0x00D8;
pub const FIOPAD_A49_REG0_OFFSET: u32 = 0x00DC;
pub const FIOPAD_C49_REG0_OFFSET: u32 = 0x00E0;
pub const FIOPAD_A51_REG0_OFFSET: u32 = 0x00E4;
pub const FIOPAD_A33_REG0_OFFSET: u32 = 0x00E8;
pub const FIOPAD_C33_REG0_OFFSET: u32 = 0x00EC;
pub const FIOPAD_C31_REG0_OFFSET: u32 = 0x00F0;
pub const FIOPAD_A31_REG0_OFFSET: u32 = 0x00F4;
pub const FIOPAD_AJ53_REG0_OFFSET: u32 = 0x00F8;
pub const FIOPAD_AL49_REG0_OFFSET: u32 = 0x00FC;
pub const FIOPAD_AL47_REG0_OFFSET: u32 = 0x0100;
pub const FIOPAD_AN49_REG0_OFFSET: u32 = 0x0104;
pub const FIOPAD_AG51_REG0_OFFSET: u32 = 0x0108;
pub const FIOPAD_AJ51_REG0_OFFSET: u32 = 0x010C;
pub const FIOPAD_AG49_REG0_OFFSET: u32 = 0x0110;
pub const FIOPAD_AE55_REG0_OFFSET: u32 = 0x0114;
pub const FIOPAD_AE53_REG0_OFFSET: u32 = 0x0118;
pub const FIOPAD_AG55_REG0_OFFSET: u32 = 0x011C;
pub const FIOPAD_AJ49_REG0_OFFSET: u32 = 0x0120;
pub const FIOPAD_AC55_REG0_OFFSET: u32 = 0x0124;
pub const FIOPAD_AC53_REG0_OFFSET: u32 = 0x0128;
pub const FIOPAD_AE51_REG0_OFFSET: u32 = 0x012C;
pub const FIOPAD_W51_REG0_OFFSET: u32 = 0x0130;
pub const FIOPAD_W55_REG0_OFFSET: u32 = 0x0134;
pub const FIOPAD_W53_REG0_OFFSET: u32 = 0x0138;
pub const FIOPAD_U55_REG0_OFFSET: u32 = 0x013C;
pub const FIOPAD_U53_REG0_OFFSET: u32 = 0x0140;
pub const FIOPAD_AE49_REG0_OFFSET: u32 = 0x0144;
pub const FIOPAD_AC49_REG0_OFFSET: u32 = 0x0148;
pub const FIOPAD_AE47_REG0_OFFSET: u32 = 0x014C;
pub const FIOPAD_AA47_REG0_OFFSET: u32 = 0x0150;
pub const FIOPAD_AA49_REG0_OFFSET: u32 = 0x0154;
pub const FIOPAD_W49_REG0_OFFSET: u32 = 0x0158;
pub const FIOPAD_AA51_REG0_OFFSET: u32 = 0x015C;
pub const FIOPAD_U49_REG0_OFFSET: u32 = 0x0160;
pub const FIOPAD_G59_REG0_OFFSET: u32 = 0x0164;
pub const FIOPAD_J59_REG0_OFFSET: u32 = 0x0168;
pub const FIOPAD_L57_REG0_OFFSET: u32 = 0x016C;
pub const FIOPAD_C59_REG0_OFFSET: u32 = 0x0170;
pub const FIOPAD_E59_REG0_OFFSET: u32 = 0x0174;
pub const FIOPAD_J57_REG0_OFFSET: u32 = 0x0178;
pub const FIOPAD_L59_REG0_OFFSET: u32 = 0x017C;
pub const FIOPAD_N59_REG0_OFFSET: u32 = 0x0180;
pub const FIOPAD_C57_REG0_OFFSET: u32 = 0x0184;
pub const FIOPAD_E57_REG0_OFFSET: u32 = 0x0188;
pub const FIOPAD_E31_REG0_OFFSET: u32 = 0x018C;
pub const FIOPAD_G31_REG0_OFFSET: u32 = 0x0190;
pub const FIOPAD_N41_REG0_OFFSET: u32 = 0x0194;
pub const FIOPAD_N39_REG0_OFFSET: u32 = 0x0198;
pub const FIOPAD_J33_REG0_OFFSET: u32 = 0x019C;
pub const FIOPAD_N33_REG0_OFFSET: u32 = 0x01A0;
pub const FIOPAD_L33_REG0_OFFSET: u32 = 0x01A4;
pub const FIOPAD_N45_REG0_OFFSET: u32 = 0x01A8;
pub const FIOPAD_N43_REG0_OFFSET: u32 = 0x01AC;
pub const FIOPAD_L31_REG0_OFFSET: u32 = 0x01B0;
pub const FIOPAD_J31_REG0_OFFSET: u32 = 0x01B4;
pub const FIOPAD_J29_REG0_OFFSET: u32 = 0x01B8;
pub const FIOPAD_E29_REG0_OFFSET: u32 = 0x01BC;
pub const FIOPAD_G29_REG0_OFFSET: u32 = 0x01C0;
pub const FIOPAD_N27_REG0_OFFSET: u32 = 0x01C4;
pub const FIOPAD_L29_REG0_OFFSET: u32 = 0x01C8;
pub const FIOPAD_J37_REG0_OFFSET: u32 = 0x01CC;
pub const FIOPAD_J39_REG0_OFFSET: u32 = 0x01D0;
pub const FIOPAD_G41_REG0_OFFSET: u32 = 0x01D4;
pub const FIOPAD_E43_REG0_OFFSET: u32 = 0x01D8;
pub const FIOPAD_L43_REG0_OFFSET: u32 = 0x01DC;
pub const FIOPAD_C43_REG0_OFFSET: u32 = 0x01E0;
pub const FIOPAD_E41_REG0_OFFSET: u32 = 0x01E4;
pub const FIOPAD_L45_REG0_OFFSET: u32 = 0x01E8;
pub const FIOPAD_J43_REG0_OFFSET: u32 = 0x01EC;
pub const FIOPAD_J41_REG0_OFFSET: u32 = 0x01F0;
pub const FIOPAD_L39_REG0_OFFSET: u32 = 0x01F4;
pub const FIOPAD_E37_REG0_OFFSET: u32 = 0x01F8;
pub const FIOPAD_E35_REG0_OFFSET: u32 = 0x01FC;
pub const FIOPAD_G35_REG0_OFFSET: u32 = 0x0200;
pub const FIOPAD_J35_REG0_OFFSET: u32 = 0x0204;
pub const FIOPAD_L37_REG0_OFFSET: u32 = 0x0208;
pub const FIOPAD_N35_REG0_OFFSET: u32 = 0x020C;
pub const FIOPAD_R51_REG0_OFFSET: u32 = 0x0210;
pub const FIOPAD_R49_REG0_OFFSET: u32 = 0x0214;
pub const FIOPAD_N51_REG0_OFFSET: u32 = 0x0218;
pub const FIOPAD_N55_REG0_OFFSET: u32 = 0x021C;
pub const FIOPAD_L55_REG0_OFFSET: u32 = 0x0220;
pub const FIOPAD_J55_REG0_OFFSET: u32 = 0x0224;
pub const FIOPAD_J45_REG0_OFFSET: u32 = 0x0228;
pub const FIOPAD_E47_REG0_OFFSET: u32 = 0x022C;
pub const FIOPAD_G47_REG0_OFFSET: u32 = 0x0230;
pub const FIOPAD_J47_REG0_OFFSET: u32 = 0x0234;
pub const FIOPAD_J49_REG0_OFFSET: u32 = 0x0238;
pub const FIOPAD_N49_REG0_OFFSET: u32 = 0x023C;
pub const FIOPAD_L51_REG0_OFFSET: u32 = 0x0240;
pub const FIOPAD_L49_REG0_OFFSET: u32 = 0x0244;
pub const FIOPAD_N53_REG0_OFFSET: u32 = 0x0248;
pub const FIOPAD_J53_REG0_OFFSET: u32 = 0x024C;

pub const FIOPAD_REG0_BEG_OFFSET: u32 = FIOPAD_AN59_REG0_OFFSET;
pub const FIOPAD_REG0_END_OFFSET: u32 = FIOPAD_J53_REG0_OFFSET;

// register offset of iopad delay
pub const FIOPAD_AJ55_REG1_OFFSET: u32 = 0x1024;
pub const FIOPAD_AL55_REG1_OFFSET: u32 = 0x1028;
pub const FIOPAD_AL53_REG1_OFFSET: u32 = 0x102C;
pub const FIOPAD_AN51_REG1_OFFSET: u32 = 0x1030;
pub const FIOPAD_AR51_REG1_OFFSET: u32 = 0x1034;
pub const FIOPAD_AJ57_REG1_OFFSET: u32 = 0x1058;
pub const FIOPAD_AG59_REG1_OFFSET: u32 = 0x105C;
pub const FIOPAD_AG57_REG1_OFFSET: u32 = 0x1060;
pub const FIOPAD_AE59_REG1_OFFSET: u32 = 0x1064;
pub const FIOPAD_BA55_REG1_OFFSET: u32 = 0x1074;
pub const FIOPAD_BA53_REG1_OFFSET: u32 = 0x1078;
pub const FIOPAD_AR59_REG1_OFFSET: u32 = 0x107C;
pub const FIOPAD_AU59_REG1_OFFSET: u32 = 0x1080;
pub const FIOPAD_A45_REG1_OFFSET: u32 = 0x10D0;
pub const FIOPAD_C45_REG1_OFFSET: u32 = 0x10D4;
pub const FIOPAD_A47_REG1_OFFSET: u32 = 0x10D8;
pub const FIOPAD_A49_REG1_OFFSET: u32 = 0x10DC;
pub const FIOPAD_C49_REG1_OFFSET: u32 = 0x10E0;
pub const FIOPAD_A51_REG1_OFFSET: u32 = 0x10E4;
pub const FIOPAD_A33_REG1_OFFSET: u32 = 0x10E8;
pub const FIOPAD_C33_REG1_OFFSET: u32 = 0x10EC;
pub const FIOPAD_C31_REG1_OFFSET: u32 = 0x10F0;
pub const FIOPAD_A31_REG1_OFFSET: u32 = 0x10F4;
pub const FIOPAD_AJ53_REG1_OFFSET: u32 = 0x10F8;
pub const FIOPAD_AL49_REG1_OFFSET: u32 = 0x10FC;
pub const FIOPAD_AL47_REG1_OFFSET: u32 = 0x1100;
pub const FIOPAD_AN49_REG1_OFFSET: u32 = 0x1104;
pub const FIOPAD_AG51_REG1_OFFSET: u32 = 0x1108;
pub const FIOPAD_AJ51_REG1_OFFSET: u32 = 0x110C;
pub const FIOPAD_AG49_REG1_OFFSET: u32 = 0x1110;
pub const FIOPAD_AE55_REG1_OFFSET: u32 = 0x1114;
pub const FIOPAD_AE53_REG1_OFFSET: u32 = 0x1118;
pub const FIOPAD_AG55_REG1_OFFSET: u32 = 0x111C;
pub const FIOPAD_AJ49_REG1_OFFSET: u32 = 0x1120;
pub const FIOPAD_AC55_REG1_OFFSET: u32 = 0x1124;
pub const FIOPAD_AC53_REG1_OFFSET: u32 = 0x1128;
pub const FIOPAD_AE51_REG1_OFFSET: u32 = 0x112C;
pub const FIOPAD_W51_REG1_OFFSET: u32 = 0x1130;
pub const FIOPAD_W53_REG1_OFFSET: u32 = 0x1138;
pub const FIOPAD_U55_REG1_OFFSET: u32 = 0x113C;
pub const FIOPAD_U53_REG1_OFFSET: u32 = 0x1140;
pub const FIOPAD_AE49_REG1_OFFSET: u32 = 0x1144;
pub const FIOPAD_AC49_REG1_OFFSET: u32 = 0x1148;
pub const FIOPAD_AE47_REG1_OFFSET: u32 = 0x114C;
pub const FIOPAD_AA47_REG1_OFFSET: u32 = 0x1150;
pub const FIOPAD_AA49_REG1_OFFSET: u32 = 0x1154;
pub const FIOPAD_W49_REG1_OFFSET: u32 = 0x1158;
pub const FIOPAD_AA51_REG1_OFFSET: u32 = 0x115C;
pub const FIOPAD_U49_REG1_OFFSET: u32 = 0x1160;
pub const FIOPAD_J59_REG1_OFFSET: u32 = 0x1168;
pub const FIOPAD_L57_REG1_OFFSET: u32 = 0x116C;
pub const FIOPAD_C59_REG1_OFFSET: u32 = 0x1170;
pub const FIOPAD_E59_REG1_OFFSET: u32 = 0x1174;
pub const FIOPAD_J57_REG1_OFFSET: u32 = 0x1178;
pub const FIOPAD_L59_REG1_OFFSET: u32 = 0x117C;
pub const FIOPAD_N59_REG1_OFFSET: u32 = 0x1180;
pub const FIOPAD_E31_REG1_OFFSET: u32 = 0x118C;
pub const FIOPAD_G31_REG1_OFFSET: u32 = 0x1190;
pub const FIOPAD_N41_REG1_OFFSET: u32 = 0x1194;
pub const FIOPAD_N39_REG1_OFFSET: u32 = 0x1198;
pub const FIOPAD_J33_REG1_OFFSET: u32 = 0x119C;
pub const FIOPAD_N33_REG1_OFFSET: u32 = 0x11A0;
pub const FIOPAD_L33_REG1_OFFSET: u32 = 0x11A4;
pub const FIOPAD_N45_REG1_OFFSET: u32 = 0x11A8;
pub const FIOPAD_N43_REG1_OFFSET: u32 = 0x11AC;
pub const FIOPAD_L31_REG1_OFFSET: u32 = 0x11B0;
pub const FIOPAD_J31_REG1_OFFSET: u32 = 0x11B4;
pub const FIOPAD_J29_REG1_OFFSET: u32 = 0x11B8;
pub const FIOPAD_E29_REG1_OFFSET: u32 = 0x11BC;
pub const FIOPAD_G29_REG1_OFFSET: u32 = 0x11C0;
pub const FIOPAD_J37_REG1_OFFSET: u32 = 0x11CC;
pub const FIOPAD_J39_REG1_OFFSET: u32 = 0x11D0;
pub const FIOPAD_G41_REG1_OFFSET: u32 = 0x11D4;
pub const FIOPAD_E43_REG1_OFFSET: u32 = 0x11D8;
pub const FIOPAD_L43_REG1_OFFSET: u32 = 0x11DC;
pub const FIOPAD_C43_REG1_OFFSET: u32 = 0x11E0;
pub const FIOPAD_E41_REG1_OFFSET: u32 = 0x11E4;
pub const FIOPAD_L45_REG1_OFFSET: u32 = 0x11E8;
pub const FIOPAD_J43_REG1_OFFSET: u32 = 0x11EC;
pub const FIOPAD_J41_REG1_OFFSET: u32 = 0x11F0;
pub const FIOPAD_L39_REG1_OFFSET: u32 = 0x11F4;
pub const FIOPAD_E37_REG1_OFFSET: u32 = 0x11F8;
pub const FIOPAD_E35_REG1_OFFSET: u32 = 0x11FC;
pub const FIOPAD_G35_REG1_OFFSET: u32 = 0x1200;
pub const FIOPAD_L55_REG1_OFFSET: u32 = 0x1220;
pub const FIOPAD_J55_REG1_OFFSET: u32 = 0x1224;
pub const FIOPAD_J45_REG1_OFFSET: u32 = 0x1228;
pub const FIOPAD_E47_REG1_OFFSET: u32 = 0x122C;
pub const FIOPAD_G47_REG1_OFFSET: u32 = 0x1230;
pub const FIOPAD_J47_REG1_OFFSET: u32 = 0x1234;
pub const FIOPAD_J49_REG1_OFFSET: u32 = 0x1238;
pub const FIOPAD_N49_REG1_OFFSET: u32 = 0x123C;
pub const FIOPAD_L51_REG1_OFFSET: u32 = 0x1240;
pub const FIOPAD_L49_REG1_OFFSET: u32 = 0x1244;
pub const FIOPAD_N53_REG1_OFFSET: u32 = 0x1248;
pub const FIOPAD_J53_REG1_OFFSET: u32 = 0x124C;

pub const FIOPAD_REG1_BEG_OFFSET: u32 = FIOPAD_AJ55_REG1_OFFSET;
pub const FIOPAD_REG1_END_OFFSET: u32 = FIOPAD_J53_REG1_OFFSET;

pub const FIOPAD_DELAY_MAX: u32 = 15;

pub const PAD_ADDRESS: u32 = 0x000_32B3_0000;
