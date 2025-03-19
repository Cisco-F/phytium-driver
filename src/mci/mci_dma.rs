#[derive(Default)]
pub struct FSdifDmaDesc {
    pub attribute: u32,
    pub non1: u32,
    pub len: u32,
    pub non2: u32,
    pub addr_lo: u32,
    pub addr_hi: u32,
    pub desc_lo: u32,
    pub desc_hi: u32,
}

pub struct FSdifDmaDescList {
    pub first_desc: *mut FSdifDmaDesc,
    pub first_desc_dma: u32,  // 第一个descriptor的物理地址
    pub desc_num: u32,
    pub desc_trans_sz: u32,   // 单个descriptor传输的字节数
}

impl Default for FSdifDmaDescList {
    fn default() -> Self {
        FSdifDmaDescList {
            first_desc: core::ptr::null_mut(),
            first_desc_dma: 0,
            desc_num: 0,
            desc_trans_sz: 0,
        }
    }
}