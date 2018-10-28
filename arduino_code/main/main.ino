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
#define CONSOLE_BUF_LEN     100
#define INCREMENT_IDX(idx)  ((idx) = ((idx) + 1 >= CONSOLE_BUF_LEN) ? 0 : ((idx) + 1))
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

static bool _servo_update_flags[ARRAY_LEN(_arm_joints)];

static console_command_t _console_commands[] = {
    {"help", _cmd_cb_help, "Print help message"},
    {"servo", _cmd_cb_servo, "Move servo to angle"},
    {"led", _cmd_cb_led, "Turn LED on or off"},
};

static char _console_buf[CONSOLE_BUF_LEN];

///////////////////////// FUNCTIONS ///////////////////////////////////
/**
 * Reads out everything currently in the buffer and checks if it is a valid command.
 */
static void _process_cmdbuffer(const uint16_t bufidx, uint16_t *readidx) {
    static char cmdbuf[MAX_COMMAND_LEN];
    static char tmpbuf[MAX_COMMAND_LEN];
    static char tokbuf[MAX_COMMAND_LEN];
    memset(cmdbuf, '\0', ARRAY_LEN(cmdbuf));
    memset(tmpbuf, '\0', ARRAY_LEN(tmpbuf));
    memset(tokbuf, '\0', ARRAY_LEN(tokbuf));

    while (*readidx < bufidx) {
        /* Move everything that we haven't examined yet from the command buffer into the tmpbuf */
        unsigned int n = MIN(bufidx - *readidx, ARRAY_LEN(tmpbuf));
        for (unsigned int i = 0; i < n; i++) {
            tmpbuf[i] = _console_buf[*readidx];
            INCREMENT_IDX(*readidx);
        }

        memcpy(tokbuf, tmpbuf, ARRAY_LEN(tokbuf));

        /* Check if tmpbuf holds a valid command */
        for (unsigned int i = 0; i < ARRAY_LEN(_console_commands); i++) {
            /* Tokenize the tokbuf by whitespace */
            char *cmd = strtok(tokbuf, " \n\r");
            if ((cmd != NULL) && (strncmp(_console_commands[i].str, cmd, MIN(bufidx, MAX_COMMAND_LEN)) == 0)) {
                _console_commands[i].func((const char *)tmpbuf, ARRAY_LEN(tmpbuf));
                break;
            }
            memcpy(tokbuf, tmpbuf, ARRAY_LEN(tokbuf));
        }
    }
}

/**
 * Checks the UART for any waiting bytes, reads them all into an internal
 * buffer, then compares all characters in that buffer up to the first
 * space, newline, or return char against each console command.
 * If there is a match, fires that command's callback synchronously.
 */
static void _check_console(void) {
    static uint16_t bufidx = 0;
    static uint16_t readidx = 0;

    /* Get any waiting bytes and read them into the circular buffer */
    bool newline_present = false;
    while (Serial.available()) {
        char c = Serial.read();
        if (c == '\n')
            newline_present = true;

        _console_buf[bufidx] = c;
        INCREMENT_IDX(bufidx);
        _console_buf[bufidx] = '\0'; // put the str term byte right after the last byte we know is valid
    }

    /* If there is a newline present, the user has finished entering a command. Check if it is valid */
    if (newline_present)
        _process_cmdbuffer(bufidx, &readidx);
}

/**
 * Adjusts each servo's angle to the value currently stored in the servo buffer.
 */
static void _manage_servos(void) {
    for (int i = 0; i < NSERVOS; i++) {
        if (_servo_update_flags[i]) {
            _arm_joints[i].servo->write(_arm_joints[i].angle);
            _servo_update_flags[i] = false;
        }
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
                Serial.println("USAGE: led <on/off>");
            }
            return;
        } else {
            return;
        }

        index++;
        tok = strtok(NULL, " \n\r");
    }
}

static void _cmd_cb_servo(const char *consolebuf, uint16_t buflen) {
    char buf[100] = {0};
    strncpy(buf, consolebuf, buflen);

    servo_id_t servoid;

    int index = 0;
    char *tok = strtok(buf, " \n\r");
    while (tok != NULL) {
        if (index == 0) {
            // Should be 'servo'
        } else if (index == 1) {
            // Parse out the servo id
            int a = atoi(tok);
            if ((a < 0) || (a >= NSERVOS)) {
                Serial.println("Illegal servo ID");
                return;
            } else {
                servoid = (servo_id_t)a;
            }
        } else if (index == 2) {
            // Parse out the angle and execute the command
            int a = atoi(tok);
            if ((a < 0) || (a > 180)) {
                Serial.println("Illegal angle");
                return;
            } else {
                _arm_joints[servoid].angle = a;
                _servo_update_flags[servoid] = true;
                return;
            }
        } else {
            return;
        }

        index++;
        tok = strtok(NULL, " ");
    }

    if (index != 3)
        Serial.println("USAGE: servo <id> <angle>");
}
