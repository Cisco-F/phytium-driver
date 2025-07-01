use alloc::vec::Vec;

#[derive(Debug,Clone)]
pub(crate) struct MCIData {
    // todo 使用智能指针涉及到大量的细微调整和代码修改，且十分容易造成潜在的非法的地址访问
    // 暂时不考虑仿照源码使用指针来表示这里的buf
    buf: Option<Vec<u32>>,
    buf_dma: usize,
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

    pub(crate) fn blksz_set(&mut self,blksz: u32) {
        self.blksz = blksz
    }

    pub(crate) fn blkcnt(&self) -> u32 {
        self.blkcnt
    }

    pub(crate) fn blkcnt_set(&mut self,blkcnt: u32) {
        self.blkcnt = blkcnt
    }

    pub(crate) fn datalen(&self) -> u32 {
        self.datalen
    }

    pub(crate) fn datalen_set(&mut self,datalen: u32) {
        self.datalen = datalen
    }

    pub(crate) fn buf(&self) -> Option<&Vec<u32>> {
        self.buf.as_ref()
    }

    pub(crate) fn buf_mut(&mut self) -> Option<&mut Vec<u32>> {
        self.buf.as_mut()
    }

    pub(crate) fn buf_take(&mut self) -> Option<Vec<u32>> {
        self.buf.take()
    }

    pub(crate) fn buf_set(&mut self,buf: Option<Vec<u32>>) {
        self.buf = buf
    }

    pub(crate) fn buf_dma(&self) -> usize {
        self.buf_dma
    }

    pub(crate) fn buf_dma_set(&mut self, buf_dma: usize) {
        self.buf_dma = buf_dma;
    }

}