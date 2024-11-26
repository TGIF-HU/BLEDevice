use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{Input, Output, Pin, PinDriver},
};

pub struct LedDriver<'d, LedPin: Pin, ButtonPin: Pin> {
    led: PinDriver<'d, LedPin, Output>,
    button: PinDriver<'d, ButtonPin, Input>,
}

impl<'d, LedPin: Pin, ButtonPin: Pin> LedDriver<'d, LedPin, ButtonPin> {
    pub fn new(
        led: PinDriver<'d, LedPin, Output>,
        button: PinDriver<'d, ButtonPin, Input>,
    ) -> Self {
        Self { led, button }
    }

    pub fn is_button_pushed(&self) -> bool {
        // Example usage of Running and Ending states
        self.button.is_high()
    }

    pub fn init(&mut self) {
        for _ in 0..10 {
            self.led.set_low().unwrap();
            FreeRtos::delay_ms(100);
            self.led.set_high().unwrap();
            FreeRtos::delay_ms(100);
        }
    }

    pub fn running(&mut self) {
        self.led.set_low().unwrap();
    }

    pub fn waiting(&mut self) {
        self.led.set_high().unwrap();
    }

    pub fn ending(&mut self) {
        for _ in 0..10 {
            self.led.toggle().unwrap();
            FreeRtos::delay_ms(200);
        }
    }
}
