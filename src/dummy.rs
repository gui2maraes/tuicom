use serialport::{DataBits, SerialPort};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DummySerial {
    buffer: VecDeque<u8>,
    baud_rate: u32,
    data_bits: DataBits,
}

impl DummySerial {
    pub fn new(baud_rate: u32) -> Self {
        Self {
            buffer: VecDeque::new(),
            baud_rate,
            data_bits: DataBits::Eight,
        }
    }
}

impl Write for DummySerial {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Read for DummySerial {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut ctr = 0;
        for i in buf {
            match self.buffer.pop_front() {
                Some(b) => *i = b,
                None => break,
            }
            ctr += 1;
        }
        Ok(ctr)
    }
}

impl SerialPort for DummySerial {
    fn baud_rate(&self) -> serialport::Result<u32> {
        Ok(self.baud_rate)
    }
    fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
        self.baud_rate = baud_rate;
        Ok(())
    }
    fn name(&self) -> Option<String> {
        Some(String::from("dummy"))
    }
    fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
        Ok(self.data_bits)
    }
    fn set_data_bits(&mut self, data_bits: serialport::DataBits) -> serialport::Result<()> {
        self.data_bits = data_bits;
        Ok(())
    }
    fn timeout(&self) -> std::time::Duration {
        Duration::from_millis(500)
    }
    fn set_timeout(&mut self, _timeout: Duration) -> serialport::Result<()> {
        Ok(())
    }
    fn clear(&self, _buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
        Ok(())
    }
    fn bytes_to_read(&self) -> serialport::Result<u32> {
        Ok(self.buffer.len() as u32)
    }
    fn bytes_to_write(&self) -> serialport::Result<u32> {
        Ok(0)
    }
    fn set_break(&self) -> serialport::Result<()> {
        Ok(())
    }
    fn clear_break(&self) -> serialport::Result<()> {
        Ok(())
    }
    fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
        Ok(serialport::FlowControl::Software)
    }
    fn set_flow_control(
        &mut self,
        _flow_control: serialport::FlowControl,
    ) -> serialport::Result<()> {
        Ok(())
    }
    fn parity(&self) -> serialport::Result<serialport::Parity> {
        todo!()
    }
    fn set_parity(&mut self, _parity: serialport::Parity) -> serialport::Result<()> {
        todo!()
    }
    fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
        todo!()
    }
    fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
        todo!()
    }
    fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
        todo!()
    }
    fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
        todo!()
    }
    fn set_stop_bits(&mut self, _stop_bits: serialport::StopBits) -> serialport::Result<()> {
        todo!()
    }
    fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
        todo!()
    }
    fn try_clone(&self) -> serialport::Result<Box<dyn SerialPort>> {
        Ok(Box::new(self.clone()))
    }
    fn write_data_terminal_ready(&mut self, _level: bool) -> serialport::Result<()> {
        todo!()
    }
    fn write_request_to_send(&mut self, _level: bool) -> serialport::Result<()> {
        todo!()
    }
}
