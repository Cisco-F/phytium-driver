use alloc::vec::Vec;

use super::constants::*;

pub struct MCIHostTransfer {
    data: Option<MCIHostData>,
    cmd: Option<MCIHostCmd>,
}

impl MCIHostTransfer {
    pub(crate) fn new() -> Self {
        MCIHostTransfer {
            data:None,
            cmd:None
        }
    }

    pub(crate) fn data(&self) -> Option<&MCIHostData> {
        self.data.as_ref()
    }

    pub(crate) fn set_data(&mut self,data:Option<MCIHostData>) {
        self.data = data
    }
    
    pub(crate) fn cmd(&self) -> Option<&MCIHostCmd> {
        self.cmd.as_ref()
    }

    pub(crate) fn set_cmd(&mut self,cmd:Option<MCIHostCmd>) {
        self.cmd = cmd
    }

    pub(crate) fn data_mut(&mut self) -> Option<&mut MCIHostData> {
        self.data.as_mut()
    }

    pub(crate) fn cmd_mut(&mut self) -> Option<&mut MCIHostCmd> {
        self.cmd.as_mut()
    }

}

#[allow(unused)]
pub(crate) struct MCIHostData {
    stream_transfer: bool,          // 指示是否为流数据传输命令
    enable_auto_command12: bool,    // 启用自动 CMD12
    enable_auto_command23: bool,    // 启用自动 CMD23
    enable_ignore_error: bool,      // 启用忽略错误以读取/写入所有数据
    data_type: u8,                  // 用于区分普通/调谐/启动数据
    block_size: usize,              // 块大小
    block_count: u32,               // 块数量
    rx_data: Option<Vec<u32>>,      // 用于保存读取数据的缓冲区
    tx_data: Option<Vec<u32>>,      // 用于写入数据的缓冲区
}

#[allow(unused)]
impl MCIHostData {

    pub(crate) fn new() -> Self {
        MCIHostData {
            stream_transfer:false,
            enable_auto_command12:false,
            enable_auto_command23:false,
            enable_ignore_error:false,
            data_type:0,
            block_size:0,
            block_count:0,
            rx_data:None,
            tx_data:None,
        }
    }

    pub(crate) fn stream_transfer(&self) -> bool {
        self.stream_transfer
    }

    pub(crate) fn enable_auto_command12(&self) -> bool {
        self.enable_auto_command12
    }

    pub(crate) fn enable_auto_command12_set(&mut self,enable_auto_command12:bool) {
        self.enable_auto_command12 = enable_auto_command12
    }

    pub(crate) fn enable_auto_command23(&self) -> bool {
        self.enable_auto_command23
    }

    pub(crate) fn enable_ignore_error(&self) -> bool {
        self.enable_ignore_error
    }

    pub(crate) fn data_type(&self) -> u8 {
        self.data_type
    }

    pub(crate) fn block_size(&self) -> usize {
        self.block_size
    }

    pub(crate) fn block_size_set(&mut self, block_size:usize) {
        self.block_size = block_size;
    } 

    pub(crate) fn block_count(&self) -> u32 {
        self.block_count
    }

    pub(crate) fn block_count_set(&mut self, block_count:u32) {
        self.block_count = block_count;
    }

    pub(crate) fn rx_data(&self) -> Option<&Vec<u32>> {
        self.rx_data.as_ref()
    }

    pub(crate) fn rx_data_set(&mut self,rx_data:Option<Vec<u32>>) {
        self.rx_data = rx_data
    }

    pub(crate) fn rx_data_mut(&mut self) -> Option<&mut Vec<u32>> {
        self.rx_data.as_mut()
    }

    pub(crate) fn rx_data_take(&mut self) -> Option<Vec<u32>> {
        self.rx_data.take()
    }

    pub(crate) fn tx_data(&self) -> Option<&Vec<u32>> {
        self.tx_data.as_ref()
    }

    pub(crate) fn tx_data_mut(&mut self) -> Option<&mut Vec<u32>> {
        self.tx_data.as_mut()
    }

    pub(crate) fn tx_data_set(&mut self,tx_data:Option<Vec<u32>>) {
        self.tx_data = tx_data
    }

    pub(crate) fn tx_data_take(&mut self) -> Option<Vec<u32>> {
        self.tx_data.take()
    }

}

#[allow(unused)]
pub(crate) struct MCIHostCmd {
    index: u32,                          // 命令索引
    argument: u32,                       // 命令参数
    cmd_type: MCIHostCmdType,            // 命令类型
    response_type: MCIHostResponseType,  // 命令响应类型
    response: [u32; 4],                  // 命令响应数据
    response_error_flags: MCIHostCardStatusFlag, // 响应错误标志
    flags: u32,                          // 命令标志
}

#[allow(unused)]
impl MCIHostCmd {

    pub(crate) fn new() -> Self {
        MCIHostCmd {
            index:0,
            argument:0,
            cmd_type:MCIHostCmdType::Normal,
            response_type:MCIHostResponseType::None,
            response:[0;4],
            response_error_flags:MCIHostCardStatusFlag::empty(),
            flags:0,
        }
    } 

    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    pub(crate) fn index_set(&mut self,index:u32) {
        self.index = index
    }

    pub(crate) fn argument(&self) -> u32 {
        self.argument
    }

    pub(crate) fn argument_set(&mut self,argument:u32) {
        self.argument = argument
    }

    pub(crate) fn cmd_type(&self) -> MCIHostCmdType {
        self.cmd_type
    }

    pub(crate) fn cmd_type_set(&mut self,cmd_type:MCIHostCmdType) {
        self.cmd_type = cmd_type
    }

    pub(crate) fn response_type(&self) -> MCIHostResponseType {
        self.response_type
    }

    pub(crate) fn response_type_set(&mut self,response_type:MCIHostResponseType) {
        self.response_type = response_type
    }

    pub(crate) fn response(&self) -> &[u32; 4] {
        &self.response
    }

    pub(crate) fn response_mut(&mut self) -> &mut [u32;4] {
        &mut self.response
    }

    pub(crate) fn response_error_flags(&self) -> MCIHostCardStatusFlag {
        self.response_error_flags
    }

    pub(crate) fn response_error_flags_set(&mut self,flags:MCIHostCardStatusFlag) {
        self.response_error_flags = flags
    }

    pub(crate) fn flags(&self) -> u32 {
        self.flags
    }
}