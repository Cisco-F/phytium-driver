use alloc::vec::Vec;

#[derive(Debug,Clone)]
pub(crate) struct MCIData {
    //TODO 使用智能指针会不会影响性能?
    buf: Option<Vec<u32>>,
    buf_dma: u32,
    blksz: u32,
    blkcnt: u32,
    datalen: u32,
}

impl MCIData {

    pub(crate) fn new() -> Self {
        MCIData {
            buf: None,
            buf_dma: 0,
            blksz: 0,
            blkcnt: 0,
            datalen: 0,
        }
    }

   pub(crate) fn blksz(&self) -> u32 {
        self.blksz
    }

    pub(crate) fn blkcnt(&self) -> u32 {
        self.blkcnt
    }

    pub(crate) fn datalen(&self) -> u32 {
        self.datalen
    }

    pub(crate) fn buf(&self) -> Option<&Vec<u32>> {
        self.buf.as_ref()
    }

    pub(crate) fn buf_mut(&mut self) -> Option<&mut Vec<u32>> {
        self.buf.as_mut()
    }

    pub(crate) fn buf_set(&mut self,buf: Option<Vec<u32>>) {
        self.buf = buf
    }

}