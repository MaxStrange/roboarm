#include <stdbool.h>
#include <stdint.h>
#include <string.h>

///////////////////////// Prototypes for Callbacks ///////////////////////////////////
static void _cmd_cb_help(const char *consolebuf, uint16_t buflen);

///////////////////////// Defines ///////////////////////////////////
#define ARRAY_LEN(a)        ((sizeof (a))/(sizeof (a)[0]))
#define CONSOLE_BUF_LEN     32
#define MAX_COMMAND_LEN     25
#define MIN(a, b)           (((a) < (b)) ? (a) : (b))
#define NSERVOS             5
#define SERVO_DEFAULT_ANGLE 90

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
} servo_t;

typedef void (*callback_t)(const char *consolebuf, uint16_t buflen);

typedef struct {
    const char *str;
    callback_t func;
    const char *description;
} console_command_t;

///////////////////////// Globals ///////////////////////////////////
static servo_t _arm_joints[NSERVOS] = {
    {SERVO_BASE, SERVO_DEFAULT_ANGLE},
    {SERVO_SHOULDER, SERVO_DEFAULT_ANGLE},
    {SERVO_ELBOW, SERVO_DEFAULT_ANGLE},
    {SERVO_WRIST, SERVO_DEFAULT_ANGLE},
    {SERVO_HAND, SERVO_DEFAULT_ANGLE},
};

console_command_t _console_commands[] = {
    {"help", _cmd_cb_help, "Print help message"},
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
    for (uint8_t i = 0; i < ARRAY_LEN(_console_commands); i++)
    {
        /* Buffer may contain a valid command plus args - take everything up to the first space or \0 */
        unsigned int n = MIN(bufidx, MAX_COMMAND_LEN);
        memcpy(tmpbuf, console_buf, sizeof(char) * n);
        char *cmd = strtok(tmpbuf, " \n\r");
        if ((cmd != NULL) && (strncmp(_console_commands[i].str, cmd, MIN(bufidx, MAX_COMMAND_LEN)) == 0))
        {
            _console_commands[i].func((const char *)console_buf, ARRAY_LEN(console_buf));
            memset(console_buf, '\0', sizeof(char) * ARRAY_LEN(console_buf));
            break;
        }
    }
}

void setup() {
    Serial.begin(115200);
}

void loop() {
    // Get a command over UART and interpret it
    _check_console();
}


///////////////////////// Command Callbacks ///////////////////////////////////
static void _cmd_cb_help(const char *consolebuf, uint16_t buflen)
{
    char buf[100] = {0};

    Serial.print("Available Commands:\n");
    for (uint16_t i = 0; i < ARRAY_LEN(_console_commands); i++)
    {
        snprintf(buf, ARRAY_LEN(buf), "%s: %s\n", _console_commands[i].str, _console_commands[i].description);
        Serial.print(buf);
    }
}
