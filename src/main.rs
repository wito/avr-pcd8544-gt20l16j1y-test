#![no_std]
#![no_main]

// Pull in the panic handler from panic-halt
extern crate panic_halt;

use arduino_uno as hardware;

use avr_hal_generic::hal::digital::*;

use hardware::prelude::*;

#[hardware::entry]
fn main() -> ! {
    let dp = hardware::Peripherals::take().unwrap();

    let mut pins = hardware::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

    // Digital pin 13 is also connected to an onboard LED marked "L"
    // let mut pin_uno_led = pins.d13.into_output(&mut pins.ddr);

    let mut pin_panel_ce = pins.d2.into_output(&mut pins.ddr);
    let mut pin_panel_reset = pins.d3.into_output(&mut pins.ddr);
    let mut pin_panel_data_select = pins.d4.into_output(&mut pins.ddr);
    let mut pin_panel_din = pins.d5.into_output(&mut pins.ddr);
    let mut pin_panel_clock = pins.d6.into_output(&mut pins.ddr);

    pin_panel_ce.set_high().void_unwrap();
    pin_panel_reset.set_high().void_unwrap();
    pin_panel_din.set_low().void_unwrap();
    pin_panel_clock.set_low().void_unwrap();

    pin_panel_ce.set_low().void_unwrap();

    reset(&mut pin_panel_reset);

    clear(
        &mut pin_panel_data_select,
        &mut pin_panel_clock,
        &mut pin_panel_din,
    );

    // display mode: Normal
    execute(
        &mut pin_panel_data_select,
        &mut pin_panel_clock,
        &mut pin_panel_din,
        0x20,
    );
    execute(
        &mut pin_panel_data_select,
        &mut pin_panel_clock,
        &mut pin_panel_din,
        0b00001100,
    );

    // bias: 40
    execute(
        &mut pin_panel_data_select,
        &mut pin_panel_clock,
        &mut pin_panel_din,
        0x21,
    );
    execute(
        &mut pin_panel_data_select,
        &mut pin_panel_clock,
        &mut pin_panel_din,
        0x14,
    );

    // contrast: 60
    execute(
        &mut pin_panel_data_select,
        &mut pin_panel_clock,
        &mut pin_panel_din,
        0x21,
    );
    execute(
        &mut pin_panel_data_select,
        &mut pin_panel_clock,
        &mut pin_panel_din,
        0xBC,
    );

    let mut pin_yj_ce = pins.d7.into_output(&mut pins.ddr);
    let mut pin_yj_dout = pins.d8;
    let mut pin_yj_din = pins.d9.into_output(&mut pins.ddr);
    let mut pin_yj_clock = pins.d10.into_output(&mut pins.ddr);

    let mut character_index: u8 = 0x0;

    loop {
        let offset: u32 = 257504;
        let character: u32 = character_index.into();
        let projected_character = character * 16 + offset;

        let bytes = projected_character.to_be_bytes();

        pin_yj_ce.set_low().void_unwrap();

        shift_out(&mut pin_yj_clock, &mut pin_yj_din, 0b00000011);
        shift_out(&mut pin_yj_clock, &mut pin_yj_din, bytes[1]);
        shift_out(&mut pin_yj_clock, &mut pin_yj_din, bytes[2]);
        shift_out(&mut pin_yj_clock, &mut pin_yj_din, bytes[3]);

        execute(
            &mut pin_panel_data_select,
            &mut pin_panel_clock,
            &mut pin_panel_din,
            0x20,
        );
        execute(
            &mut pin_panel_data_select,
            &mut pin_panel_clock,
            &mut pin_panel_din,
            128,
        );
        execute(
            &mut pin_panel_data_select,
            &mut pin_panel_clock,
            &mut pin_panel_din,
            64,
        );

        for _ in 0..8 {
            send(
                &mut pin_panel_data_select,
                &mut pin_panel_clock,
                &mut pin_panel_din,
                shift_in(&mut pin_yj_clock, &mut pin_yj_dout),
            );
        }

        execute(
            &mut pin_panel_data_select,
            &mut pin_panel_clock,
            &mut pin_panel_din,
            0x20,
        );
        execute(
            &mut pin_panel_data_select,
            &mut pin_panel_clock,
            &mut pin_panel_din,
            128,
        );
        execute(
            &mut pin_panel_data_select,
            &mut pin_panel_clock,
            &mut pin_panel_din,
            65,
        );

        for _ in 0..8 {
            send(
                &mut pin_panel_data_select,
                &mut pin_panel_clock,
                &mut pin_panel_din,
                shift_in(&mut pin_yj_clock, &mut pin_yj_dout),
            );
        }

        execute(
            &mut pin_panel_data_select,
            &mut pin_panel_clock,
            &mut pin_panel_din,
            0x20,
        );
        execute(
            &mut pin_panel_data_select,
            &mut pin_panel_clock,
            &mut pin_panel_din,
            128,
        );
        execute(
            &mut pin_panel_data_select,
            &mut pin_panel_clock,
            &mut pin_panel_din,
            66,
        );

        pin_yj_ce.set_high().void_unwrap();

        character_index = character_index + 1;

        hardware::delay_ms(250);
    }
}

fn set_for_data<T: v2::OutputPin>(data_control_pin: &mut T) {
    let _ = data_control_pin.set_high();
}

fn set_for_command<T: v2::OutputPin>(data_control_pin: &mut T) {
    let _ = data_control_pin.set_low();
}

fn shift_out<T: v2::OutputPin, U: v2::OutputPin>(clock_pin: &mut T, output_pin: &mut U, value: u8) {
    for i in 0..8 {
        let _ = clock_pin.set_low();

        if (value & (1 << (7 - i))) != 0 {
            let _ = output_pin.set_high();
        } else {
            let _ = output_pin.set_low();
        }

        let _ = clock_pin.set_high();
    }
}

fn shift_in<T: v2::OutputPin, U: v2::InputPin>(clock_pin: &mut T, input_pin: &mut U) -> u8 {
    let mut value = 0;

    for i in 0..8 {
        let _ = clock_pin.set_low();
        let _ = clock_pin.set_high();

        if input_pin.is_high().unwrap_or(false) {
            value = value | (1 << (7 - i))
        }
    }

    return value;
}

fn reset<T: v2::OutputPin>(reset_pin: &mut T) {
    let _ = reset_pin.set_low();
    let _ = reset_pin.set_high();
}

fn clear<T: v2::OutputPin, U: v2::OutputPin, V: v2::OutputPin>(
    data_control_pin: &mut T,
    clock_pin: &mut U,
    output_pin: &mut V,
) {
    for _ in 0..504 {
        send(data_control_pin, clock_pin, output_pin, 0x0);
    }

    execute(data_control_pin, clock_pin, output_pin, 0x20);
    execute(data_control_pin, clock_pin, output_pin, 0x40);
    execute(data_control_pin, clock_pin, output_pin, 0x80);

    send(data_control_pin, clock_pin, output_pin, 0);
}

fn send<T: v2::OutputPin, U: v2::OutputPin, V: v2::OutputPin>(
    data_control_pin: &mut T,
    clock_pin: &mut U,
    output_pin: &mut V,
    data: u8,
) {
    set_for_data(data_control_pin);
    shift_out(clock_pin, output_pin, data);
}

fn execute<T: v2::OutputPin, U: v2::OutputPin, V: v2::OutputPin>(
    data_control_pin: &mut T,
    clock_pin: &mut U,
    output_pin: &mut V,
    command: u8,
) {
    set_for_command(data_control_pin);
    shift_out(clock_pin, output_pin, command);
}
