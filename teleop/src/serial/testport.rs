use serialport;
use serialport::Result;
use std::time::Duration;
use std::io;

pub struct TestPort;

const DEFAULT_BAUD_RATE: u32 = 115200;

impl serialport::SerialPort for TestPort {
    fn name(&self) -> Option<String> {
        Some("Test Port".to_string())
    }

    fn settings(&self) -> serialport::SerialPortSettings {
        serialport::SerialPortSettings {
            baud_rate:      DEFAULT_BAUD_RATE,
            data_bits:      serialport::DataBits::Eight,
            flow_control:   serialport::FlowControl::None,
            parity:         serialport::Parity::None,
            stop_bits:      serialport::StopBits::One,
            timeout:        Duration::from_millis(30),
        }
    }

    fn baud_rate(&self) -> Result<u32> {
        Ok(DEFAULT_BAUD_RATE)
    }

    fn data_bits(&self) -> Result<serialport::DataBits> {
        Ok(serialport::DataBits::Eight)
    }

    fn flow_control(&self) -> Result<serialport::FlowControl> {
        Ok(serialport::FlowControl::None)
    }

    fn parity(&self) -> Result<serialport::Parity> {
        Ok(serialport::Parity::None)
    }

    fn stop_bits(&self) -> Result<serialport::StopBits> {
        Ok(serialport::StopBits::One)
    }

    fn timeout(&self) -> Duration {
        Duration::from_millis(30)
    }

    fn set_all(&mut self, _settings: &serialport::SerialPortSettings) -> Result<()> {
        Ok(())
    }

    fn set_baud_rate(&mut self, _baud_rate: u32) -> Result<()> {
        Ok(())
    }

    fn set_data_bits(&mut self, _data_bits: serialport::DataBits) -> Result<()> {
        Ok(())
    }

    fn set_flow_control(&mut self, _flow_control: serialport::FlowControl) -> Result<()> {
        Ok(())
    }

    fn set_parity(&mut self, _parity: serialport::Parity) -> Result<()> {
        Ok(())
    }

    fn set_stop_bits(&mut self, _stop_bits: serialport::StopBits) -> Result<()> {
        Ok(())
    }

    fn set_timeout(&mut self, _timeout: Duration) -> Result<()> {
        Ok(())
    }

    fn write_request_to_send(&mut self, _level: bool) -> Result<()> {
        Ok(())
    }

    fn write_data_terminal_ready(&mut self, _level: bool) -> Result<()> {
        Ok(())
    }

    fn read_clear_to_send(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn read_data_set_ready(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn read_ring_indicator(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn read_carrier_detect(&mut self) -> Result<bool> {
        Ok(true)
    }

    fn try_clone(&self) -> Result<Box<serialport::SerialPort>> {
        Ok(Box::new(TestPort))
    }
}

impl io::Write for TestPort {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl io::Read for TestPort {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut n = 0;
        for (i, c) in "test".as_bytes().iter().enumerate() {
            buf[i] = *c;
            n += 1;
        }
        Ok(n)
    }
}
