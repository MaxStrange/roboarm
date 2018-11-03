#include <Servo.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>

///////////////////////// Prototypes for Callbacks ///////////////////////////////////
static void _cmd_cb_help(const char *consolebuf, uint16_t buflen);
static void _cmd_cb_led(const char *consolebuf, uint16_t buflen);
static void _cmd_cb_servo(const char *consolebuf, uint16_t buflen);
static void _cmd_cb_home(const char *consolebuf, uint16_t buflen);

///////////////////////// Defines ///////////////////////////////////
/* Some useful macros */
#define ARRAY_LEN(a)                ((sizeof (a))/(sizeof (a)[0]))
#define INCREMENT_IDX(idx)          ((idx) = ((idx) + 1 >= CONSOLE_BUF_LEN) ? 0 : ((idx) + 1))
#define MIN(a, b)                   (((a) < (b)) ? (a) : (b))

/** The length of the console */
#define CONSOLE_BUF_LEN             100

/** The maximum allowed command length */
#define MAX_COMMAND_LEN             25

/** The number of servos on the robot arm */
#define NSERVOS                     5

/* Default angles (starting angles and angles the arm homes to) */
#define DEFAULT_ANGLE_BASE          90
#define DEFAULT_ANGLE_SHOULDER      10
#define DEFAULT_ANGLE_ELBOW         155
#define DEFAULT_ANGLE_WRIST         90
#define DEFAULT_ANGLE_HAND          90

/* Pin numbers for servos */
#define PIN_SERVO_BASE              3
#define PIN_SERVO_SHOULDER          5
#define PIN_SERVO_ELBOW             6
#define PIN_SERVO_WRIST             9
#define PIN_SERVO_HAND              10
#define PIN_LED                     13

/* Limits to keep the robot from destroying itself - empirically determined */
#define LOWER_LIMIT_ANGLE_BASE      0
#define UPPER_LIMIT_ANGLE_BASE      180
#define LOWER_LIMIT_ANGLE_SHOULDER  0
#define UPPER_LIMIT_ANGLE_SHOULDER  50
#define LOWER_LIMIT_ANGLE_ELBOW     100
#define UPPER_LIMIT_ANGLE_ELBOW     180
#define LOWER_LIMIT_ANGLE_WRIST     80
#define UPPER_LIMIT_ANGLE_WRIST     100
#define LOWER_LIMIT_ANGLE_HAND      0
#define UPPER_LIMIT_ANGLE_HAND      180

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
    uint16_t lower_limit;
    uint16_t upper_limit;
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
    {SERVO_BASE, DEFAULT_ANGLE_BASE, &__s0, PIN_SERVO_BASE, LOWER_LIMIT_ANGLE_BASE, UPPER_LIMIT_ANGLE_BASE},
    {SERVO_SHOULDER, DEFAULT_ANGLE_SHOULDER, &__s1, PIN_SERVO_SHOULDER, LOWER_LIMIT_ANGLE_SHOULDER, UPPER_LIMIT_ANGLE_SHOULDER},
    {SERVO_ELBOW, DEFAULT_ANGLE_ELBOW, &__s2, PIN_SERVO_ELBOW, LOWER_LIMIT_ANGLE_ELBOW, UPPER_LIMIT_ANGLE_ELBOW},
    {SERVO_WRIST, DEFAULT_ANGLE_WRIST, &__s3, PIN_SERVO_WRIST, LOWER_LIMIT_ANGLE_WRIST, UPPER_LIMIT_ANGLE_WRIST},
    {SERVO_HAND, DEFAULT_ANGLE_HAND, &__s4, PIN_SERVO_HAND, LOWER_LIMIT_ANGLE_HAND, UPPER_LIMIT_ANGLE_HAND},
};

static bool _servo_update_flags[ARRAY_LEN(_arm_joints)];

static console_command_t _console_commands[] = {
    {"help", _cmd_cb_help, "Print help message"},
    {"servo", _cmd_cb_servo, "Move servo to angle"},
    {"led", _cmd_cb_led, "Turn LED on or off"},
    {"home", _cmd_cb_home, "Move all servos to home location"},
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

    Serial.println("Newline present. Processing command.");
    Serial.print("  -> Readidx: "); Serial.println(*readidx);
    Serial.print("  -> bufidx:  "); Serial.println(bufidx);

    while (*readidx != bufidx) {
        /* Move everything that we haven't examined yet from the command buffer into the tmpbuf */

        /* If we are not on the same lap, we need to unroll the buffer into tmpbuf appropriately */
        unsigned int n;
        if (*readidx > bufidx)
            n = bufidx + (ARRAY_LEN(_console_buf) - *readidx);
        else
            n = bufidx - *readidx;

        // Make sure we don't overrun our tmpbuf
        n = MIN(n, ARRAY_LEN(tmpbuf));

        /* Now copy everything into the tmpbuf */
        for (unsigned int i = 0; i < n; i++) {
            tmpbuf[i] = _console_buf[*readidx];
            INCREMENT_IDX(*readidx);
        }

        Serial.println("  -> Our tmpbuf looks like:");
        Serial.println(tmpbuf);

        /* Now copy everything from tmpbuf into tokbuf, which we will use to tokenize */
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
        Serial.print("Add to idx "); Serial.println(bufidx);
        INCREMENT_IDX(bufidx);
        Serial.print("Put null at "); Serial.println(bufidx);
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

/**
 * Set servo with ID `id` to `angle` to be moved on the next iteration of the loop.
 */
static void _servo_goto(servo_id_t id, uint16_t angle) {
    static char buf[100];
    snprintf(buf, ARRAY_LEN(buf), "Sending %d to %d", id, angle);
    Serial.println(buf);
    _arm_joints[id].angle = angle;
    _servo_update_flags[id] = true;
}

void setup() {
    // Set up the LED
    pinMode(PIN_LED, OUTPUT);

    // Set up the UART
    Serial.begin(115200);

    // Set up all the joints
    for (int i = 0; i < NSERVOS; i++) {
        _arm_joints[i].servo->attach(_arm_joints[i].pin);
    }

    // Reset all servos to their home locations
    _cmd_cb_home("", 0);
}

void loop() {
    // Get a command over UART and interpret it
    _check_console();

    // Move any servos that have their update-pending flag set
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
            if ((a < _arm_joints[servoid].lower_limit) || (a > _arm_joints[servoid].upper_limit)) {
                Serial.println("Illegal angle");
                Serial.print("Angle for id "); Serial.print(id); Serial.print(" should be between ");
                Serial.print(_arm_joints[servoid].lower_limt); Serial.print(" and ");
                Serial.println(_arm_joints[servoid].upper_limit);
                return;
            } else {
                _servo_goto(servoid, a);
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

static void _cmd_cb_home(const char *consolebuf, uint16_t buflen) {
    _servo_goto(SERVO_BASE, DEFAULT_ANGLE_BASE);
    _servo_goto(SERVO_ELBOW, DEFAULT_ANGLE_ELBOW);
    _servo_goto(SERVO_HAND, DEFAULT_ANGLE_HAND);
    _servo_goto(SERVO_SHOULDER, DEFAULT_ANGLE_SHOULDER);
    _servo_goto(SERVO_WRIST, DEFAULT_ANGLE_WRIST);
}
