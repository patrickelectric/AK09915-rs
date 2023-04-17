use embedded_hal::blocking::i2c::{Write, WriteRead};
use embedded_hal::blocking::delay::DelayUs;

const AK09915_ADDRESS: u8 = 0x0C;

// Register Addresses
const AK09915_REG_WIA2: u8 = 0x01; // Device ID Register 2
const AK09915_REG_ST1: u8 = 0x10; // Data Status Register 1
const AK09915_REG_HXL: u8 = 0x11; // Data Register - X-axis Magnetic Data Low Byte
const AK09915_REG_WIA1: u8 = 0x00; // Device ID Register 1
const AK09915_REG_HXH: u8 = 0x12; // Data Register - X-axis Magnetic Data High Byte
const AK09915_REG_HYL: u8 = 0x13; // Data Register - Y-axis Magnetic Data Low Byte
const AK09915_REG_HYH: u8 = 0x14; // Data Register - Y-axis Magnetic Data High Byte
const AK09915_REG_HZL: u8 = 0x15; // Data Register - Z-axis Magnetic Data Low Byte
const AK09915_REG_HZH: u8 = 0x16; // Data Register - Z-axis Magnetic Data High Byte
const AK09915_REG_TMPS: u8 = 0x17; // Temperature Sensor Data Register
const AK09915_REG_ST2: u8 = 0x18; // Data Status Register 2
const AK09915_REG_CNTL2: u8 = 0x31; // Control Register 2
const AK09915_REG_CNTL3: u8 = 0x32; // Control Register 3
const AK09915_REG_TS1: u8 = 0x33; // Self Test Register 1
const AK09915_REG_TS2: u8 = 0x34; // Self Test Register 2
const AK09915_REG_I2CDIS: u8 = 0x3A; // I2C Disable Register

// AK09915 Mode Settings - Corresponding to Control Register 2
const AK09915_MODE_POWERDOWN: u8 = 0x00;
const AK09915_MODE_SINGLE: u8 = 0x01;
const AK09915_MODE_CONTINUOUS_10HZ: u8 = 0x02;
const AK09915_MODE_CONTINUOUS_20HZ: u8 = 0x04;
const AK09915_MODE_CONTINUOUS_50HZ: u8 = 0x06;
const AK09915_MODE_CONTINUOUS_100HZ: u8 = 0x08;
const AK09915_MODE_CONTINUOUS_200HZ: u8 = 0x0A;
const AK09915_MODE_CONTINUOUS_1HZ: u8 = 0x0C;
const AK09915_MODE_SELFTEST: u8 = 0x10;

pub struct Ak09915<I2C> {
    pub i2c: I2C,
    pub address: u8,
}

pub enum Mode {
    PowerDown,
    SingleMeasurement,
    ContMeasurement10,
    ContMeasurement20,
    ContMeasurement50,
    ContMeasurement100,
    ContMeasurement200,
    ContMeasurement1,
    SelfTest,
}


impl<I2C, E> Ak09915<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    pub fn new(i2c: I2C) -> Self {
        Self { i2c , address : AK09915_ADDRESS }
    }

    fn write_register(&mut self, register: u8, value: u8) -> Result<(), E> {
        self.i2c
            .write(self.address, &[register, value])
    }

    fn read_register(&mut self, register: u8) -> Result<u8, E> {
        let mut buffer = [0u8];
        self.i2c
            .write_read(self.address, &[register], &mut buffer)
            .and(Ok(buffer[0]))
    }

    pub fn init(&mut self) -> Result<(), E> {
        // Soft reset device and put on continuous measurement
        self.reset()?;
        self.set_mode(Mode::ContMeasurement50)?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), E> {
        // Soft reset device
        self.write_register(AK09915_REG_CNTL3, 0x01)?;
        Ok(())
    }

    pub fn set_mode(&mut self, mode: Mode) -> Result<(), E> {
        let reg = match mode {
            Mode::PowerDown => AK09915_MODE_POWERDOWN,
            Mode::SingleMeasurement => AK09915_MODE_SINGLE,
            Mode::ContMeasurement10 => AK09915_MODE_CONTINUOUS_10HZ,
            Mode::ContMeasurement20 => AK09915_MODE_CONTINUOUS_20HZ,
            Mode::ContMeasurement50 => AK09915_MODE_CONTINUOUS_50HZ,
            Mode::ContMeasurement100 => AK09915_MODE_CONTINUOUS_100HZ,
            Mode::ContMeasurement200 => AK09915_MODE_CONTINUOUS_200HZ,
            Mode::ContMeasurement1 => AK09915_MODE_CONTINUOUS_1HZ,
            Mode::SelfTest => AK09915_MODE_SELFTEST,
        };
        //When user wants to change operation mode,
        //transit to power-down mode first and then transit to other modes. After Power-down mode is set, at least 100
        //µs (Twait) is needed before setting another mode.
        self.write_register(AK09915_REG_CNTL2, AK09915_MODE_POWERDOWN)?;
        
        //not working, dirty solution
        // let mut delay = DelayUs::
        // delay.delay_us(100u32);
        std::thread::sleep(std::time::Duration::from_micros(100));

        self.write_register(AK09915_REG_CNTL2, reg)?;
        Ok(())
    }

    // 9.4.4.1. Self-test Sequence:
    //   1. Set Power-down mode (MODE[4:0] bits = "00000").
    //   2. Set Self-test mode (MODE[4:0] bits = "10000").
    //   3. Check Data Ready by:
    //      - Polling DRDY bit of ST1 register.
    //      - Monitoring DRDY pin.
    //      When Data Ready, proceed to the next step.
    //   4. Read measurement data (HXL to HZH).
    // 9.4.4.2. Self-test Judgment:
    //   If measurement data read by the above sequence is within the following ranges,
    //   AK09915 is working normally:
    //     - HX[15:0] bits: -200 ≤ HX ≤ +200
    //     - HY[15:0] bits: -200 ≤ HY ≤ +200
    //     - HZ[15:0] bits: -800 ≤ HZ ≤ -200

    pub fn self_test(&mut self) -> Result<(), E> {
        self.set_mode(Mode::SelfTest)?;

        self.is_data_ready()?;

        let (hx, hy, hz) = self.read_mag()?;

        // Self-test judgment
        if (hx >= -200 && hx <= 200) && (hy >= -200 && hy <= 200) && (hz >= -800 && hz <= -200) {
            println!("Self-test passed \nMagnetometer: x={}, y={}, z={}", hx, hy, hz);
        } else {
            println!("Self-test failed");
        }
        Ok(())
    }

    pub fn is_data_ready(&mut self) -> Result<bool, E> {
        let mut retries = 10;
        while retries > 0 {
            let status = self.read_register(AK09915_REG_ST1)?;
            if (status & 0x01) != 0 {
                return Ok(true); // Data ready
            }
            std::thread::sleep(std::time::Duration::from_micros(100));
            retries -= 1;
        }
        Ok(false) // Data not ready after retries
    }

    pub fn read_mag(&mut self) -> Result<(i16, i16, i16), E> {
        let mut buffer: [u8; 6] = [0u8; 6];
        self.i2c.write_read(self.address, &[AK09915_REG_HXL], &mut buffer)?;
        let x = i16::from_le_bytes([buffer[0], buffer[1]]);
        let y = i16::from_le_bytes([buffer[2], buffer[3]]);
        let z = i16::from_le_bytes([buffer[4], buffer[5]]);
        Ok((x, y, z))
    }
}