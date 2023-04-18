use ak09915_rs::Ak09915;
use ak09915_rs::Mode;
use linux_embedded_hal::I2cdev;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Receive the i2c device as parameter
    #[arg(short, long, default_value_t = ("/dev/i2c-1").to_string())]
    device: String,
}

fn main() {

    let args = Args::parse();
    let dev = I2cdev::new(args.device).unwrap();
    let mut sensor = Ak09915::new(dev);

    if sensor.self_test().unwrap(){
        println!("Self test -  OK");     
    }

    println!("Test 5 single measurement, and 5 without set single measurement(test data ready"); 
    for _n in 1..=5 {
        sensor.set_mode(Mode::Single).unwrap();
        if sensor.is_data_ready().unwrap(){
            let (x, y, z) = sensor.read_mag().unwrap();
            println!("Magnetometer: x={}, y={}, z={}", x, y, z);
        }
        }
    println!("Test 5 measurement, without set single measurement(test data ready");        
    for _n in 1..=5 {
        if sensor.is_data_ready().unwrap(){
            let (x, y, z) = sensor.read_mag().unwrap();
            println!("Magnetometer: x={}, y={}, z={}", x, y, z);
        }
        }
    println!("Test 5 measurement, using continuous mode"); 
    sensor.set_mode(Mode::Cont200Hz).unwrap();
    for _n in 1..=5 {
        if sensor.is_data_ready().unwrap(){
            let (x, y, z) = sensor.read_mag().unwrap();
            println!("Magnetometer: x={}, y={}, z={}", x, y, z);
        }
        std::thread::sleep(std::time::Duration::from_micros(1000));
        }
}