#[cfg(target_os = "linux")] 
use embedded_hal::i2c::I2c;
#[cfg(target_os = "linux")] 
use linux_embedded_hal::{I2cdev, I2CError};
use std::sync::{Arc, Mutex};

const I2C_BUS_PATH: &str = "/dev/i2c-1";

const MAX17048_I2C_ADDRESS: u8 = 0b0110110;
const ACCELEROMETER_I2C_ADDRESS: u8 = 0b0011000; 
const EEPROM_I2C_ADDRESS_MEMORY: u8 = 0b1010000;

#[derive(Clone)]
pub struct I2cInterface {
    device: Arc<Mutex<I2cDevice>>
}

impl I2cInterface {
    pub fn new() -> Self {
        let i2c = loop {
            match Self::try_init_i2c() {
                Ok(i2c) => break i2c,
                Err(_) => std::thread::sleep(std::time::Duration::from_secs(1))
            }
        };
        Self { device: Arc::new(Mutex::new(i2c)) }
    }

    fn try_init_i2c() -> Result<I2cDevice, I2CError> {
        let mut i2c = I2cDevice::new(I2C_BUS_PATH)?;        
        // Init Accelerometer (LIS3DH)
        // CTRL_REG1 (0x20): Enable all axes, 50 Hz data rate
        // 0x4F = 0b01001111: ODR=0100 (50Hz), Z=Y=X=1 (all axes enabled)
        i2c.write(ACCELEROMETER_I2C_ADDRESS, &[0x20, 0x4F])?;
        // CTRL_REG4 (0x23): 4g range, high resolution mode
        // 0x98 = 0b10011000: BDU=1, FS=01 (4g range), HR=1 (high resolution)
        i2c.write(ACCELEROMETER_I2C_ADDRESS, &[0x23, 0x98])?;

        Ok(i2c)
    }

    pub async fn read_accelerometer_data(&self) -> (i16, i16, i16) {
        let bytes = self.read_async::<6>(ACCELEROMETER_I2C_ADDRESS, 0x28 | 0x80).await;
        let x = i16::from_le_bytes([bytes[0], bytes[1]]) >> 4;
        let y = i16::from_le_bytes([bytes[2], bytes[3]]) >> 4;
        let z = i16::from_le_bytes([bytes[4], bytes[5]]) >> 4;
        (x, y, z)
    }

    async fn read_async<const N: usize>(
        &self,
        address: u8,
        register: u8,
    ) -> [u8; N] {
        let device = Arc::clone(&self.device);
        tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; N];
            if let Ok(mut device) = device.lock() {
                let _ = device.write_read(address, &[register], &mut buf);
            }
            buf
        })
        .await
        .unwrap_or([0u8; N])
    }
}

#[cfg(not(target_os = "linux"))]
pub struct I2CError;

#[cfg(target_os = "linux")]
type I2cDevice = I2cdev;

#[cfg(not(target_os = "linux"))]
type I2cDevice = MockI2cdev;

// Mock I2C device for non-Linux platforms (Windows development)
#[cfg(not(target_os = "linux"))]
pub struct MockI2cdev {
    memory: [u8; 8192]
}

#[cfg(not(target_os = "linux"))]
impl MockI2cdev {
    pub fn new(_path: &str) -> Result<Self, I2CError> {
        Ok(MockI2cdev {
            memory: [0; 8192]
        })
    }
    
    pub fn write(&mut self, _addr: u8, _data: &[u8]) -> Result<(), I2CError> {
        self.memory[.._data.len()].copy_from_slice(_data);
        Ok(())
    }
    
    pub fn write_read(&mut self, _addr: u8, _write_data: &[u8], _read_data: &mut [u8]) -> Result<(), I2CError> {
        self.memory[.._write_data.len()].copy_from_slice(_write_data);
        _read_data.copy_from_slice(&self.memory[.._read_data.len()]);
        Ok(())
    }
}
