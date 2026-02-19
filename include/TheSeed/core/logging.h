// Datastructures
typedef enum {
  TS_LOG_DEBUG,
  TS_LOG_INFO,
  TS_LOG_WARN,
  TS_LOG_ERROR,
  TS_LOG_CRITICAL,
} TS_Log_Level;

typedef struct {
  TS_Log_Level level;
  char *msg;
} TS_Log_Entry;

// Logging functions

void ts_debug(char *, ...);
void ts_info(char *, ...);
void ts_warn(char *, ...);
void ts_error(char *, ...);
void ts_critical(char *, ...);

#define ts_assert(exp, fmt, ...)                                               \
  if (!(exp)) {                                                                \
    ts_critical(fmt __VA_OPT__(, ) __VA_ARGS__);                               \
  }

#define ts_assert_recoverable(exp, fmt, ...)                                   \
  if (!(exp)) {                                                                \
    ts_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                  \
  }

#define ts_assert_abort(exp, fmt, ...)                                         \
  if (!(exp)) {                                                                \
    ts_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                  \
    return;                                                                    \
  }

#define ts_assert_abort_value(exp, value, fmt, ...)                            \
  if (!(exp)) {                                                                \
    ts_error(fmt __VA_OPT__(, ) __VA_ARGS__);                                  \
    return value;                                                              \
  }

// Logger registry

typedef void (*TS_Logger)(TS_Log_Entry);

void ts_add_logger(TS_Logger);

// Default loggers

void ts_stdout_logger(TS_Log_Entry entry);

// Init
void ts_logging_init();
