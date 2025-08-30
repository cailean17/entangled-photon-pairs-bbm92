
use serialport::{SerialPort, available_ports};
use rand::{Rng, thread_rng, rngs::ThreadRng};

const BASIS_BLOCK_BIT_SIZE : u32 = 65536;
const SLOT_PERIOD: u32 = 1_000_000;
pub struct BasisStreamGenerator {
  uart_port: Option<Box<dyn SerialPort>> 
}

impl BasisStreamGenerator{

  pub fn new(uart_port: Option<Box<dyn SerialPort>>) -> Self{
      Self { uart_port }
  }

  /**
    * Function Def: prints a list of active port connections on agent device
    * Input: N/A
    * Ouput: N/A
    */
  pub fn obtain_available_port_lists(&self){
    match available_ports() {
      Ok(p) => {
        println!("Found {} ports", p.len());
        for port in p{
          println!(" {}", port.port_name)
        }
      },
      Err(e) => println!(" Error obtaining available ports list{}", e),
    };
  }

  pub fn open_serial_port(&mut self, port_name : String, baud_rate: u32){
    match serialport::new(port_name, baud_rate).open() {
      Ok(port) => {
          self.uart_port = Some(port);
      },
      Err(error) => {
          println!("Error when trying to open serial port: {}", error);
          self.uart_port = None;
      }
    };

  }

  pub fn generate_basis_blocks(&self){
    let mut rng: ThreadRng = thread_rng();
    let mut basis_block : Vec<u8> = Vec::new();
    for _i in 0..BASIS_BLOCK_BIT_SIZE {
      let random : u8 = rng.gen_range(0..=1);
      basis_block.push(random);
    }
    println!("Basis Block {:#?}", basis_block);
  }
}