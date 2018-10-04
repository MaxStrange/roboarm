#include <Servo.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

///////////////////////// Prototypes for Callbacks ///////////////////////////////////
static void _cmd_cb_help(const char *consolebuf, uint16_t buflen);
static void _cmd_cb_led(const char *consolebuf, uint16_t buflen);
static void _cmd_cb_servo(const char *consolebuf, uint16_t buflen);

///////////////////////// Defines ///////////////////////////////////
#define ARRAY_LEN(a)        ((sizeof (a))/(sizeof (a)[0]))
#define CONSOLE_BUF_LEN     32
#define MAX_COMMAND_LEN     25
#define MIN(a, b)           (((a) < (b)) ? (a) : (b))
#define NSERVOS             5
#define SERVO_DEFAULT_ANGLE 90
#define PIN_SERVO_BASE      3
#define PIN_SERVO_SHOULDER  5
#define PIN_SERVO_ELBOW     6
#define PIN_SERVO_WRIST     9
#define PIN_SERVO_HAND      10
#define PIN_LED             13

///////////////////////// Typedefs ///////////////////////////////////
typedef enum {
    SERVO_BASE,
    SERVO_SHOULDER,
    SERVO_ELBOW,
    SERVO_WRIST,
    SERVO_HAND,
} servo_id_t;

typedef struct {
    servo_id_t id;
    uint16_t angle;
    Servo *servo;
    uint8_t pin;
} my_servo_t;

typedef void (*callback_t)(const char *consolebuf, uint16_t buflen);

typedef struct {
    const char *str;
    callback_t func;
    const char *description;
} console_command_t;

///////////////////////// Globals ///////////////////////////////////
Servo __s0;
Servo __s1;
Servo __s2;
Servo __s3;
Servo __s4;

static my_servo_t _arm_joints[NSERVOS] = {
    {SERVO_BASE, SERVO_DEFAULT_ANGLE, &__s0, PIN_SERVO_BASE},
    {SERVO_SHOULDER, SERVO_DEFAULT_ANGLE, &__s1, PIN_SERVO_SHOULDER},
    {SERVO_ELBOW, SERVO_DEFAULT_ANGLE, &__s2, PIN_SERVO_ELBOW},
    {SERVO_WRIST, SERVO_DEFAULT_ANGLE, &__s3, PIN_SERVO_WRIST},
    {SERVO_HAND, SERVO_DEFAULT_ANGLE, &__s4, PIN_SERVO_HAND},
};

console_command_t _console_commands[] = {
    {"help", _cmd_cb_help, "Print help message"},
    {"servo", _cmd_cb_servo, "Move servo to angle"},
    {"led", _cmd_cb_led, "Turn LED on or off"},
};

///////////////////////// FUNCTIONS ///////////////////////////////////
/**
 * Checks the UART for any waiting bytes, reads them all into an internal
 * buffer, then compares all characters in that buffer up to the first
 * space, newline, or return char against each console command.
 * If there is a match, fires that command's callback synchronously.
 */
static void _check_console(void) {
    static uint8_t console_buf[CONSOLE_BUF_LEN];
    static uint16_t bufidx = 0;

    char cmdbuf[MAX_COMMAND_LEN];
    char tmpbuf[MAX_COMMAND_LEN];

    /* Get the next byte */
    while (Serial.available()) {
        console_buf[bufidx++] = Serial.read();

        // Prevent overflow
        if (bufidx >= CONSOLE_BUF_LEN)
            bufidx = 0;

        console_buf[bufidx] = '\0'; // put the str term byte right after the last byte we know is valid
    }

    /* Check if the buffer holds a valid command */
    for (uint8_t i = 0; i < ARRAY_LEN(_console_commands); i++) {
        /* Buffer may contain a valid command plus args - take everything up to the first space or \0 */
        unsigned int n = MIN(bufidx, MAX_COMMAND_LEN);
        memcpy(tmpbuf, console_buf, sizeof(char) * n);
        char *cmd = strtok(tmpbuf, " \n\r");
        if ((cmd != NULL) && (strncmp(_console_commands[i].str, cmd, MIN(bufidx, MAX_COMMAND_LEN)) == 0)) {
            _console_commands[i].func((const char *)console_buf, ARRAY_LEN(console_buf));
            memset(console_buf, '\0', sizeof(char) * ARRAY_LEN(console_buf));
            break;
        }
    }
}

/**
 * Adjusts each servo's angle to the value currently stored in the servo buffer.
 */
static void _manage_servos(void) {
    for (int i = 0; i < NSERVOS; i++) {
        _arm_joints[i].servo->write(_arm_joints[i].angle);
    }
}

void setup() {
    pinMode(PIN_LED, OUTPUT);
    Serial.begin(115200);

    for (int i = 0; i < NSERVOS; i++) {
        _arm_joints[i].servo->attach(_arm_joints[i].pin);
    }
}

void loop() {
    // Get a command over UART and interpret it
    _check_console();
    _manage_servos();
}


///////////////////////// Command Callbacks ///////////////////////////////////
static void _cmd_cb_help(const char *consolebuf, uint16_t buflen) {
    char buf[100] = {0};

    Serial.print("Available Commands:\n");
    for (uint16_t i = 0; i < ARRAY_LEN(_console_commands); i++) {
        snprintf(buf, ARRAY_LEN(buf), "%s: %s\n", _console_commands[i].str, _console_commands[i].description);
        Serial.print(buf);
    }
}

static void _cmd_cb_led(const char *consolebuf, uint16_t buflen) {
    char buf[100] = {0};
    strncpy(buf, consolebuf, buflen);

    int index = 0;
    char *tok = strtok(buf, " ");
    while (tok != NULL) {
        if (index == 0) {
            // Should be 'led'
        } else if (index == 1) {
            if (strncmp(tok, "on", ARRAY_LEN(buf)) == 0) {
                digitalWrite(PIN_LED, HIGH);
            } else if (strncmp(tok, "off", ARRAY_LEN(buf)) == 0) {
                digitalWrite(PIN_LED, LOW);
            } else {
                Serial.print("USAGE: led <on/off>");
            }
            return;
        } else {
            return;
        }

        index++;
        tok = strtok(buf, " ");
    }
}

static void _cmd_cb_servo(const char *consolebuf, uint16_t buflen) {
    char buf[100] = {0};
    strncpy(buf, consolebuf, buflen);

    servo_id_t servoid;

    int index = 0;
    char *tok = strtok(buf, " ");
    while (tok != NULL) {
        if (index == 0) {
            // Should be 'servo'
        } else if (index == 1) {
            // Parse out the servo id
            int a = atoi(tok);
            if ((a < 0) || (a >= NSERVOS)) {
                Serial.print("Illegal servo ID");
                return;
            } else {
                servoid = (servo_id_t)a;
            }
        } else if (index == 2) {
            // Parse out the angle and execute the command
            int a = atoi(tok);
            if ((a < 0) || (a > 360)) {
                Serial.print("Illegal angle");
                return;
            } else {
                _arm_joints[servoid].angle = a;
                return;
            }
        } else {
            return;
        }

        index++;
        tok = strtok(buf, " ");
    }
}
