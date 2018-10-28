#include <Servo.h>

#define PIN_DUT     9
#define DELAY_MS    250

Servo dut;

void setup() {
    dut.attach(PIN_DUT);
    Serial.begin(115200);
}

void loop() {
    for (int pos = 0; pos <= 180; pos += 10) {
        dut.write(pos);
        delay(DELAY_MS);
        Serial.print(pos);
        Serial.print("\n\r");
    }

    for (int pos = 180; pos >= 0; pos -= 10) {
        dut.write(pos);
        delay(DELAY_MS);
        Serial.print(pos);
        Serial.print("\n\r");
    }
}