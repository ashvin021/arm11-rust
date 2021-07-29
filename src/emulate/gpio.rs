const GPIO_10: usize = 0x20200000;
const GPIO_20: usize = 0x20200004;
const GPIO_30: usize = 0x20200008;
const PIN_OFF: usize = 0x20200028;
const PIN_ON: usize = 0x2020001c;

pub fn gpio_accessed(mem_address: usize) -> bool {
    match mem_address {
        GPIO_10 | GPIO_20 | GPIO_30 | PIN_OFF | PIN_ON => true,
        _ => false,
    }
}

pub fn print_gpio_message(mem_address: usize) {
    match mem_address {
        GPIO_10 => println!("One GPIO pin from 0 to 9 has been accessed"),
        GPIO_20 => println!("One GPIO pin from 10 to 19 has been accessed"),
        GPIO_30 => println!("One GPIO pin from 20 to 29 has been accessed"),
        PIN_OFF => println!("PIN OFF"),
        PIN_ON => println!("PIN ON"),
        _ => panic!("Invalid gpio address - can't print message."),
    }
}
